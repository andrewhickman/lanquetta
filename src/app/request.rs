use std::string::ToString;

use druid::{
    widget::{prelude::*, Controller, TextBox},
    Data, Lens, Widget, WidgetExt,
};

use crate::app::command;
use crate::json::JsonText;
use crate::widget::{FormField, ValidationState};
use crate::{grpc, protobuf, theme};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: ValidationState<JsonText, grpc::Request, Option<String>>,
    proto: Option<protobuf::ProtobufRequest>,
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
    pub(in crate::app) fn request(&self) -> grpc::Request {
        grpc::Request { body: todo!() }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            body: ValidationState::new(JsonText::default(), Err(None)),
            proto: None,
        }
    }
}

fn request_validator(descriptor: Option<protobuf::ProtobufRequest>) -> RequestValidator {
    Box::new(move |s| match &descriptor {
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
            if let Some(method) = command.get(command::SELECT_METHOD) {
                child.set_validate(request_validator(Some(method.request())), data);
                data.set_raw(method.request().empty_json().into());
            }
        }
        child.event(ctx, event, data, env)
    }
}
