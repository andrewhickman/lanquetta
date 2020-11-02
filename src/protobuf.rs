mod codec;
mod file;

pub use self::{
    codec::ProtobufCodec,
    file::{ProtobufMethod, ProtobufService},
};

use anyhow::Result;
use druid::Data;
use protobuf::{json, reflect::MessageDescriptor, MessageDyn};

pub fn to_json(message: &dyn MessageDyn) -> Result<String> {
    json::print_to_string_with_options(
        message,
        &json::PrintOptions {
            proto_field_name: true,
            always_output_default_values: true,
            ..json::PrintOptions::default()
        },
    )
    .map_err(|err| anyhow::format_err!("{:?}", err))
}

#[derive(Copy, Clone, Data, Debug, Eq, PartialEq)]
pub enum ProtobufMethodKind {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct ProtobufMessage {
    descriptor: MessageDescriptor,
}

impl ProtobufMessage {
    pub fn new(descriptor: MessageDescriptor) -> Self {
        ProtobufMessage { descriptor }
    }

    pub fn parse(&self, s: &str) -> Result<Box<dyn MessageDyn>> {
        Ok(json::parse_dynamic_from_str_with_options(
            &self.descriptor,
            s,
            &protobuf::json::ParseOptions {
                ignore_unknown_fields: false,
                ..Default::default()
            },
        )?)
    }

    pub fn empty(&self) -> Box<dyn MessageDyn> {
        self.descriptor.new_instance()
    }

    pub fn empty_json(&self) -> String {
        to_json(&*self.empty()).unwrap_or_default()
    }
}

impl druid::Data for ProtobufMessage {
    fn same(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor
    }
}
