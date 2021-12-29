use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, Either, Flex, Label, TextBox},
    Data, Lens, Widget, WidgetExt as _,
};
use prost_reflect::{DynamicMessage, MessageDescriptor, ReflectMessage};

use crate::{
    grpc,
    json::JsonText,
    theme,
    widget::{Empty, FormField, ValidationState, FINISH_EDIT},
};

type RequestValidationState = ValidationState<JsonText, grpc::Request, String>;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
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

    Flex::column()
        .with_flex_child(textbox, 1.0)
        .with_child(error)
        .lens(State::body)
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
                data.text_mut().prettify();
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
        if field.is_list() {
            let value = make_template_field(field.kind());
            message.set_field(field.number(), prost_reflect::Value::List(vec![value]));
        } else if field.is_map() {
            let map_entry = field.kind();
            let map_entry = map_entry.as_message().unwrap();

            let key = prost_reflect::MapKey::default_value(&map_entry.get_field(1).unwrap().kind());
            let value = make_template_field(map_entry.get_field(2).unwrap().kind());

            message.set_field(
                field.number(),
                prost_reflect::Value::Map([(key, value)].into()),
            );
        } else {
            message.set_field(field.number(), make_template_field(field.kind()));
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
