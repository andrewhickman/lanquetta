mod codec;
mod file;

pub use self::codec::ProtobufCodec;
pub use self::file::ProtobufFile;

use std::sync::Arc;

use anyhow::Result;
use protobuf::MessageDyn;
use protobuf::reflect::MessageDescriptor;

#[derive(Debug, Clone)]
pub struct ProtobufRequest {
    descriptor: MessageDescriptor,
}

impl ProtobufRequest {
    pub fn parse(&self, s: &str) -> Result<Arc<dyn MessageDyn>> {
        let item = protobuf::json::parse_dynamic_from_str_with_options(&self.descriptor, s, &protobuf::json::ParseOptions {
            ignore_unknown_fields: false,
            ..Default::default()
        })?;
        Ok(item.into())
    }
}

impl druid::Data for ProtobufRequest {
    fn same(&self, other: &Self) -> bool {
        self.descriptor == other.descriptor
    }
}
