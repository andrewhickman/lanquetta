use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, CrossAxisAlignment, Either, Flex, Label, TextBox},
    Data, Lens, Point, Widget, WidgetExt as _, WidgetPod,
};
use prost_reflect::{DynamicMessage, MessageDescriptor, ReflectMessage};

use crate::{
    app::metadata,
    grpc,
    json::JsonText,
    theme,
    widget::{Empty, FormField, ValidationState, FINISH_EDIT},
};

type RequestValidationState = ValidationState<JsonText, grpc::Request, String>;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    metadata: metadata::State,
    body: RequestValidationState,
}

struct RequestController;

pub(in crate::app) fn build() -> impl Widget<State> {
    let textbox = FormField::new(theme::text_box_scope(
        TextBox::multiline().with_font(theme::EDITOR_FONT),
    ))
    .controller(RequestController)
    .expand();
    let error_label =
        theme::error_label_scope(Label::dynamic(|data: &RequestValidationState, _| {
            data.result().err().cloned().unwrap_or_default()
        }));
    let error = Either::new(
        |data: &RequestValidationState, _| !data.is_pristine_or_valid(),
        error_label,
        Empty,
    )
    .expand_width();

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
    metadata: WidgetPod<metadata::State, Box<dyn Widget<metadata::State>>>,
}

impl State {
    pub fn empty(request: prost_reflect::MessageDescriptor) -> Self {
        let json = make_template_message_json(request.clone());
        State::with_text(request, json)
    }

    pub fn with_text(request: prost_reflect::MessageDescriptor, json: impl Into<JsonText>) -> Self {
        State {
            metadata: metadata::State::default(),
            body: ValidationState::dirty(
                json.into(),
                Arc::new(move |s| {
                    grpc::Request::from_json(request.clone(), s).map_err(|e| e.to_string())
                }),
            ),
        }
    }

    pub(in crate::app) fn is_valid(&self) -> bool {
        self.body.is_valid()
    }

    pub(in crate::app) fn get(&self) -> Option<&grpc::Request> {
        self.body.result().ok()
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
        let metadata_bc = BoxConstraints::new(
            Size::new(bc.min().width, 0.0),
            Size::new(bc.max().width, bc.max().height / 2.0),
        );
        let metadata_size = self.metadata.layout(ctx, &metadata_bc, &data.metadata, env);

        let remaining_height = bc.max().height - metadata_size.height;
        let body_bc = BoxConstraints::new(
            Size::new(bc.min().width, remaining_height),
            Size::new(bc.max().width, remaining_height),
        );
        let body_size = self.body.layout(ctx, &body_bc, &data.body, env);

        self.body.set_origin(ctx, Point::ZERO);
        self.metadata
            .set_origin(ctx, Point::new(0.0, body_size.height));

        Size::new(
            metadata_size.width.max(body_size.width),
            metadata_size.height + body_size.height,
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
