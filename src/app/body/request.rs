use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, Either, Flex, Label, TextBox},
    Data, Lens, Widget, WidgetExt as _,
};

use crate::{
    grpc,
    json::JsonText,
    protobuf::ProtobufMessage,
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
    pub fn empty(request: ProtobufMessage) -> Self {
        let json = JsonText::pretty(request.empty_json());
        State::with_text(request, json)
    }

    pub fn with_text(request: ProtobufMessage, json: impl Into<JsonText>) -> Self {
        State {
            body: ValidationState::dirty(
                json.into(),
                Arc::new(move |s| match request.parse(s) {
                    Ok(body) => Ok(grpc::Request { body }),
                    Err(err) => Err(err.to_string()),
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
