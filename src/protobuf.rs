mod codec;
mod file;

pub use self::codec::ProtobufCodec;
pub use self::file::{ProtobufMethod, ProtobufService};

use std::sync::Arc;

use anyhow::Result;
use protobuf::json;
use protobuf::reflect::MessageDescriptor;
use protobuf::MessageDyn;

#[derive(Debug, Clone)]
pub struct ProtobufRequest {
    descriptor: MessageDescriptor,
}

impl ProtobufRequest {
    pub fn new(descriptor: MessageDescriptor) -> Self {
        ProtobufRequest { descriptor }
    }

    #[allow(unused)]
    pub fn parse(&self, s: &str) -> Result<Arc<dyn MessageDyn>> {
        let item = json::parse_dynamic_from_str_with_options(
            &self.descriptor,
            s,
            &protobuf::json::ParseOptions {
                ignore_unknown_fields: false,
                ..Default::default()
            },
        )?;
        Ok(item.into())
    }

    pub fn empty_json(&self) -> String {
        let empty = self.descriptor.new_instance();
        json::print_to_string_with_options(
            &*empty,
            &json::PrintOptions {
                proto_field_name: true,
                always_output_default_values: true,
                ..json::PrintOptions::default()
            },
        )
        .unwrap_or_default()
    }
}

impl druid::Data for ProtobufRequest {
    fn same(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor
    }
}
