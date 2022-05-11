use prost_reflect::prost::Message;
use prost_reflect::{DynamicMessage, MethodDescriptor, ReflectMessage};
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};

use crate::grpc;

#[derive(Debug, Clone)]
pub struct DynamicCodec(MethodDescriptor);

impl DynamicCodec {
    pub fn new(desc: MethodDescriptor) -> Self {
        DynamicCodec(desc)
    }
}

impl Codec for DynamicCodec {
    type Encode = grpc::Request;
    type Decode = grpc::Response;

    type Encoder = DynamicCodec;
    type Decoder = DynamicCodec;

    fn encoder(&mut self) -> Self::Encoder {
        self.clone()
    }

    fn decoder(&mut self) -> Self::Decoder {
        self.clone()
    }
}

impl Encoder for DynamicCodec {
    type Item = grpc::Request;
    type Error = Status;

    fn encode(&mut self, request: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        debug_assert_eq!(request.message.descriptor(), self.0.input());
        request
            .message
            .encode(dst)
            .expect("insufficient space for message");
        Ok(())
    }
}

impl Decoder for DynamicCodec {
    type Item = grpc::Response;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let mut message = DynamicMessage::new(self.0.output());
        message
            .merge(src)
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Some(grpc::Response::new(message)))
    }
}
