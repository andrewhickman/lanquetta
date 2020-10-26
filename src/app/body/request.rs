use std::sync::Arc;

use druid::{
    widget::{prelude::*, Controller, TextBox},
    Data, Lens, Widget, WidgetExt as _,
};

use crate::{
    app::command,
    grpc,
    json::JsonText,
    protobuf::{ProtobufMethod},
    theme,
    widget::{FormField, ValidationState},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: ValidationState<JsonText, grpc::Request, String>,
}

struct RequestController;

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    FormField::new(TextBox::multiline().with_font(theme::EDITOR_FONT).expand())
        .controller(RequestController)
        .lens(State::body)
        .boxed()
}

impl State {
    pub(in crate::app) fn new(method: ProtobufMethod) -> Self {
        let mut json = JsonText::from(method.request().empty_json());
        json.prettify();

        let request = method.request();

        State {
            body: ValidationState::new(
                json,
                Arc::new(move |s| match request.parse(s) {
                    Ok(body) => Ok(grpc::Request { method: method.clone(), body }),
                    Err(err) => Err(err.to_string()),
                }),
            ),
        }
    }

    pub(in crate::app) fn get(&self) -> Option<&grpc::Request> {
        self.body.result().ok()
    }
}

impl<W> Controller<ValidationState<JsonText, grpc::Request, String>, FormField<W>>
    for RequestController
where
    W: Widget<JsonText>,
{
    fn event(
        &mut self,
        child: &mut FormField<W>,
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
        child: &mut FormField<W>,
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
