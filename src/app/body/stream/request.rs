use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, Either, Flex, Label, TextBox},
    Data, Lens, Widget, WidgetExt as _,
};

use crate::{
    grpc,
    json::JsonText,
    protobuf::ProtobufMethod,
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
    .expand_width();
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
        .with_child(textbox)
        .with_child(error)
        .lens(State::body)
}

impl State {
    pub fn empty(method: ProtobufMethod) -> Self {
        let json = JsonText::pretty(method.request().empty_json());
        State::with_text(method, json)
    }

    pub fn with_text(method: ProtobufMethod, json: impl Into<JsonText>) -> Self {
        let request = method.request();

        State {
            body: ValidationState::dirty(
                json.into(),
                Arc::new(move |s| match request.parse(s) {
                    Ok(body) => Ok(grpc::Request {
                        method: method.clone(),
                        body,
                    }),
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