use druid::{
    widget::{prelude::*, Controller, TextBox},
    Data, Lens, Widget, WidgetExt,
};

use crate::{
    app::command,
    grpc,
    json::JsonText,
    protobuf::{self, ProtobufMethod},
    theme,
    widget::{FormField, ValidationState},
};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: ValidationState<JsonText, grpc::Request, Option<String>>,
    proto: protobuf::ProtobufRequest,
}

struct RequestController;

type RequestValidator = Box<dyn Fn(&str) -> Result<grpc::Request, Option<String>>>;

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    FormField::new(
        TextBox::multiline().with_font(theme::EDITOR_FONT).expand(),
        request_validator(None),
    )
    .controller(RequestController)
    .lens(State::body)
    .boxed()
}

impl State {
    pub(in crate::app) fn new(method: &ProtobufMethod) -> Self {
        State {
            body: ValidationState::new(JsonText::from(method.request().empty_json()), Err(None)),
            proto: method.request(),
        }
    }

    pub(in crate::app) fn request(&self) -> grpc::Request {
        grpc::Request { body: todo!() }
    }
}

fn request_validator(descriptor: Option<protobuf::ProtobufRequest>) -> RequestValidator {
    // TODO
    Box::new(move |_| match &descriptor {
        // Some(descriptor) => match descriptor.parse(s) {
        //     Ok(body) => Ok(grpc::Request { body }),
        //     Err(err) => Err(Some(err.to_string())),
        // },
        // None => Err(None),
        _ => Err(None),
    })
}

impl<W>
    Controller<
        ValidationState<JsonText, grpc::Request, Option<String>>,
        FormField<W, RequestValidator>,
    > for RequestController
where
    W: Widget<JsonText>,
{
    fn event(
        &mut self,
        child: &mut FormField<W, RequestValidator>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut ValidationState<JsonText, grpc::Request, Option<String>>,
        env: &Env,
    ) {
        if let Event::Command(command) = event {
            if command.is(command::FORMAT) {
                data.raw_mut().prettify();
            }
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut FormField<W, RequestValidator>,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &ValidationState<JsonText, grpc::Request, Option<String>>,
        env: &Env,
    ) {
        if let LifeCycle::FocusChanged(false) = event {
            ctx.submit_command(command::FORMAT.to(ctx.widget_id()));
        }

        child.lifecycle(ctx, event, data, env)
    }
}
