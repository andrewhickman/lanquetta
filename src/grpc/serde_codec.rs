use std::string::ToString;

use bytes::buf::ext::{BufExt, BufMutExt};
use protobuf::Message;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::Status;

#[derive(Default, Debug)]
pub struct SerdeCodec;

pub struct SerdeEncoder;
pub struct SerdeDecoder;

impl Codec for SerdeCodec {
    type Encode = <SerdeEncoder as Encoder>::Item;
    type Decode = <SerdeDecoder as Decoder>::Item;

    type Encoder = SerdeEncoder;
    type Decoder = SerdeDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        SerdeEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        SerdeDecoder
    }
}

impl Encoder for SerdeEncoder {
    type Item = protobuf::well_known_types::Value;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf) -> Result<(), Self::Error> {
        item.write_to_writer(&mut dst.writer())
            .expect("bytes operations are infallible");
        Ok(())
    }
}

impl Decoder for SerdeDecoder {
    type Item = protobuf::well_known_types::Value;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf) -> Result<Option<Self::Item>, Self::Error> {
        let item = protobuf::parse_from_reader(&mut src.reader())
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        Ok(Some(item))
    }
}
