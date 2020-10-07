use std::string::ToString;

use druid::{Data, Lens, Widget, WidgetExt};

use crate::widget::{FormField, TextArea, ValidationState};
use crate::{grpc, protobuf};

#[derive(Debug, Clone, Data, Lens)]
pub(in crate::app) struct State {
    body: ValidationState<grpc::Request, Option<String>>,
    proto: Option<protobuf::ProtobufRequest>,
}

pub(in crate::app) fn build() -> impl Widget<State> {
    let text_area = TextArea::new().styled();
    FormField::new(text_area, request_validator(None))
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
            body: ValidationState::new(String::new(), Err(None)),
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
