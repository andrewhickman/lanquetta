use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, CrossAxisAlignment, Either, Flex, Label, TextBox},
    Data, Lens, Widget, WidgetExt as _,
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
    let error_label = theme::error_label_scope(Label::dynamic(|data: &State, _| {
        data.body.result().err().cloned().unwrap_or_default()
    }));
    let error = Either::new(
        |data: &State, _| !data.body.is_pristine_or_valid(),
        error_label,
        Empty,
    )
    .expand_width();

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_flex_child(textbox.lens(State::body), 1.0)
        .with_child(error)
        .with_default_spacer()
        .with_child(metadata::build_editable().lens(State::metadata))
}

pub(in crate::app) fn build_header() -> impl Widget<State> {
    Label::new("Request editor")
        .with_font(theme::font::HEADER_TWO)
        .align_left()
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
