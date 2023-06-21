use std::sync::Arc;

use druid::{
    piet::TextStorage,
    widget::{prelude::*, Controller, CrossAxisAlignment, Flex, Label},
    ArcStr, Data, Insets, Lens, Point, Widget, WidgetExt as _, WidgetPod,
};
use prost_reflect::{DynamicMessage, MessageDescriptor, ReflectMessage};
use tonic::metadata::MetadataMap;

use crate::{
    app::metadata,
    grpc,
    json::JsonText,
    lens,
    theme::{self, BODY_SPACER},
    widget::{code_area, error_label, FormField, ValidationState, FINISH_EDIT},
};

type RequestValidationState = ValidationState<JsonText, grpc::Request, ArcStr>;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    metadata: metadata::EditableState,
    body: RequestValidationState,
}

struct RequestController;

pub(in crate::app) fn build() -> impl Widget<State> {
    let textbox = FormField::text_box(code_area(true))
        .controller(RequestController)
        .expand();
    let error = error_label(Insets::ZERO)
        .expand_width()
        .lens(lens::Project::new(|data: &RequestValidationState| {
            data.error()
        }));

    let body = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(textbox, 1.0)
        .with_child(error);
    let metadata = metadata::build_editable();

    RequestLayout {
        body: WidgetPod::new(body.boxed()),
        metadata: WidgetPod::new(metadata.boxed()),
    }
}

pub(in crate::app) fn build_header() -> impl Widget<State> {
    Label::new("Request editor")
        .with_font(theme::font::HEADER_TWO)
        .align_left()
}

struct RequestLayout {
    body: WidgetPod<RequestValidationState, Box<dyn Widget<RequestValidationState>>>,
    metadata: WidgetPod<metadata::EditableState, Box<dyn Widget<metadata::EditableState>>>,
}

impl State {
    pub fn empty(request: prost_reflect::MessageDescriptor, metadata: metadata::State) -> Self {
        let json = make_template_message_json(request.clone());
        State::with_text(request, json, metadata)
    }

    pub fn with_text(
        request: prost_reflect::MessageDescriptor,
        json: impl Into<JsonText>,
        metadata: metadata::State,
    ) -> Self {
        State {
            metadata: metadata::EditableState::new(metadata),
            body: ValidationState::dirty(
                json.into(),
                Arc::new(move |s| {
                    grpc::Request::from_json(request.clone(), s.as_str())
                        .map_err(|e| e.to_string().into())
                }),
            ),
        }
    }

    pub(in crate::app) fn is_valid(&self) -> bool {
        self.body.is_valid() && self.metadata.is_valid()
    }

    pub(in crate::app) fn get(&self) -> Option<&grpc::Request> {
        self.body.result().ok()
    }

    pub(in crate::app) fn tonic_metadata(&self) -> MetadataMap {
        self.metadata.metadata()
    }

    pub(in crate::app) fn serde_metadata(&self) -> metadata::State {
        self.metadata.to_state()
    }

    pub(in crate::app) fn get_json(&self) -> &JsonText {
        self.body.text()
    }

    pub fn text(&self) -> &JsonText {
        self.body.text()
    }
}

impl Widget<State> for RequestLayout {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        self.body.event(ctx, event, &mut data.body, env);
        self.metadata.event(ctx, event, &mut data.metadata, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
        self.body.lifecycle(ctx, event, &data.body, env);
        self.metadata.lifecycle(ctx, event, &data.metadata, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &State, data: &State, env: &Env) {
        self.body.update(ctx, &data.body, env);
        self.metadata.update(ctx, &data.metadata, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &State,
        env: &Env,
    ) -> Size {
        let body_spacer = if data.metadata.is_empty() {
            0.0
        } else {
            BODY_SPACER
        };

        let metadata_bc = BoxConstraints::new(
            Size::new(bc.min().width, 0.0),
            Size::new(
                bc.max().width,
                (bc.max().height - body_spacer).max(bc.min().height) / 2.0,
            ),
        );
        let metadata_size = self.metadata.layout(ctx, &metadata_bc, &data.metadata, env);

        let remaining_height =
            (bc.max().height - body_spacer - metadata_size.height).max(bc.min().height);
        let body_bc = BoxConstraints::new(
            Size::new(bc.min().width, remaining_height),
            Size::new(bc.max().width, remaining_height),
        );
        let body_size = self.body.layout(ctx, &body_bc, &data.body, env);

        self.body.set_origin(ctx, Point::ZERO);
        self.metadata
            .set_origin(ctx, Point::new(0.0, body_size.height + body_spacer));

        Size::new(
            metadata_size.width.max(body_size.width),
            metadata_size.height + body_size.height + body_spacer,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        self.body.paint(ctx, &data.body, env);
        self.metadata.paint(ctx, &data.metadata, env);
    }
}

impl Controller<RequestValidationState, FormField<JsonText>> for RequestController {
    fn event(
        &mut self,
        child: &mut FormField<JsonText>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut RequestValidationState,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(FINISH_EDIT) {
                data.with_text_mut(|t| t.prettify())
            }
        }
        child.event(ctx, event, data, env)
    }
}

fn make_template_message_json(desc: MessageDescriptor) -> JsonText {
    let message = make_template_message(desc);

    JsonText::pretty(grpc::Response::new(message).to_json())
}

fn make_template_message(desc: MessageDescriptor) -> DynamicMessage {
    let mut message = DynamicMessage::new(desc);

    for field in message.descriptor().fields() {
        match message.get_field_mut(&field) {
            prost_reflect::Value::List(ref mut list) => {
                list.push(make_template_field(field.kind()));
            }
            prost_reflect::Value::Map(ref mut map) => {
                let kind = field.kind();
                let entry = kind.as_message().unwrap();
                let key = prost_reflect::MapKey::default_value(&entry.map_entry_key_field().kind());
                let value = make_template_field(entry.map_entry_value_field().kind());
                map.insert(key, value);
            }
            _ => (),
        }
    }

    message
}

fn make_template_field(kind: prost_reflect::Kind) -> prost_reflect::Value {
    match kind {
        prost_reflect::Kind::Message(message) => {
            prost_reflect::Value::Message(make_template_message(message))
        }
        kind => prost_reflect::Value::default_value(&kind),
    }
}
