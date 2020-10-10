use std::string::ToString;

use druid::widget::TextBox;
use druid::{Data, Lens, Widget, WidgetExt};

use crate::widget::{FormField, ValidationState};
use crate::{grpc, protobuf};
use crate::json::JsonText;

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: ValidationState<JsonText, grpc::Request, Option<String>>,
    proto: Option<protobuf::ProtobufRequest>,
}

pub(in crate::app) fn build() -> Box<dyn Widget<State>> {
    FormField::new(TextBox::multiline().expand(), request_validator(None))
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

fn request_validator(
    descriptor: Option<protobuf::ProtobufRequest>,
) -> impl Fn(&str) -> Result<grpc::Request, Option<String>> {
    move |s| match &descriptor {
        Some(descriptor) => match descriptor.parse(s) {
            Ok(body) => Ok(grpc::Request { body }),
            Err(err) => Err(Some(err.to_string())),
        },
        None => Err(None),
    }
}
