use bytes::buf::ext::{BufExt, BufMutExt};
use protobuf::reflect::MessageDescriptor;
use protobuf::{CodedInputStream, CodedOutputStream, MessageDyn};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::Status;

#[derive(Default, Debug)]
pub struct ProtobufCodec {
    descriptor: Option<MessageDescriptor>,
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
            descriptor: Some(descriptor),
        }
    }
}

impl Codec for ProtobufCodec {
    type Encode = <ProtobufEncoder as Encoder>::Item;
    type Decode = <ProtobufDecoder as Decoder>::Item;

    type Encoder = ProtobufEncoder;
    type Decoder = ProtobufDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        ProtobufEncoder {
            descriptor: self.descriptor.clone().unwrap(),
        }
    }

    fn decoder(&mut self) -> Self::Decoder {
        ProtobufDecoder {
            descriptor: self.descriptor.clone().unwrap(),
        }
    }
}

impl Encoder for ProtobufEncoder {
    type Item = Box<dyn MessageDyn>;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf) -> Result<(), Self::Error> {
        item.write_to_dyn(&mut CodedOutputStream::new(&mut dst.writer()))
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        Ok(())
    }
}

impl Decoder for ProtobufDecoder {
    type Item = Box<dyn MessageDyn>;
    type Error = Status;

    fn decode(&mut self, dst: &mut DecodeBuf) -> Result<Option<Self::Item>, Self::Error> {
        let mut item = self.descriptor.new_instance();
        item.merge_from_dyn(&mut CodedInputStream::new(&mut dst.reader()))
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        Ok(Some(item))
    }
}
