use std::sync::Arc;

use bytes::buf::ext::{BufExt, BufMutExt};
use protobuf::reflect::MessageDescriptor;
use protobuf::{CodedInputStream, CodedOutputStream, MessageDyn};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::Status;

#[derive(Debug)]
pub struct ProtobufCodec {
    descriptor: MessageDescriptor,
}

pub struct ProtobufEncoder {
    descriptor: MessageDescriptor,
}

pub struct ProtobufDecoder {
    descriptor: MessageDescriptor,
}

impl ProtobufCodec {
    pub fn new(descriptor: MessageDescriptor) -> Self {
        ProtobufCodec {
            descriptor,
        }
    }
}

impl Default for ProtobufCodec {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Codec for ProtobufCodec {
    type Encode = <ProtobufEncoder as Encoder>::Item;
    type Decode = <ProtobufDecoder as Decoder>::Item;

    type Encoder = ProtobufEncoder;
    type Decoder = ProtobufDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        ProtobufEncoder {
            descriptor: self.descriptor.clone(),
        }
    }

    fn decoder(&mut self) -> Self::Decoder {
        ProtobufDecoder {
            descriptor: self.descriptor.clone(),
        }
    }
}

impl Encoder for ProtobufEncoder {
    type Item = Arc<dyn MessageDyn>;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf) -> Result<(), Self::Error> {
        item.write_to_dyn(&mut CodedOutputStream::new(&mut dst.writer()))
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        Ok(())
    }
}

impl Decoder for ProtobufDecoder {
    type Item = Arc<dyn MessageDyn>;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf) -> Result<Option<Self::Item>, Self::Error> {
        let mut item = self.descriptor.new_instance();
        item.merge_from_dyn(&mut CodedInputStream::new(&mut src.reader()))
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        Ok(Some(item.into()))
    }
}
