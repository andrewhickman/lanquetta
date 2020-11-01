use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, TextBox},
    Data, Lens, Widget, WidgetExt as _,
};

use crate::{
    app::command,
    grpc,
    json::JsonText,
    protobuf::ProtobufMethod,
    theme,
    widget::{FormField, ValidationState},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: ValidationState<JsonText, grpc::Request, String>,
}

struct RequestController;

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    FormField::new(theme::text_box_scope(
        TextBox::multiline().with_font(theme::EDITOR_FONT),
    ))
    .controller(RequestController)
    .expand()
    .lens(State::body)
    .boxed()
}

impl State {
    pub fn empty(method: ProtobufMethod) -> Self {
        let json = JsonText::pretty(method.request().empty_json());
        State::with_text(method, json)
    }

    pub fn with_text(method: ProtobufMethod, json: impl Into<JsonText>) -> Self {
        let request = method.request();

        State {
            body: ValidationState::new(
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

<<<<<<< HEAD
impl<W> Controller<ValidationState<JsonText, grpc::Request, String>, FormField<JsonText, W>>
    for RequestController
=======
impl<W> Controller<RequestValidationState, FormField<JsonText, W>> for RequestController
>>>>>>> 9dc168f (fixup! use widgetpod in formfield)
where
    W: Widget<JsonText>,
{
    fn event(
        &mut self,
        child: &mut FormField<JsonText, W>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut ValidationState<JsonText, grpc::Request, String>,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(command::FORMAT) {
                data.with_text_mut(JsonText::prettify);
            }
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut FormField<JsonText, W>,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ValidationState<JsonText, grpc::Request, String>,
        env: &Env,
    ) {
        if let LifeCycle::FocusChanged(false) = event {
            ctx.submit_command(command::FORMAT.to(ctx.widget_id()));
        }

        child.lifecycle(ctx, event, data, env)
    }
}
