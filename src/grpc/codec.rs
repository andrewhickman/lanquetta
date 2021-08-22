use bytes::{Buf, BufMut};
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};

use super::{Request, Response};

#[derive(Debug, Default)]
pub struct BytesCodec;

impl Codec for BytesCodec {
    type Encode = Request;
    type Decode = Response;

    type Encoder = BytesCodec;
    type Decoder = BytesCodec;

    fn encoder(&mut self) -> Self::Encoder {
        BytesCodec
    }

    fn decoder(&mut self) -> Self::Decoder {
        BytesCodec
    }
}

impl Encoder for BytesCodec {
    type Item = Request;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put(item.bytes);
        Ok(())
    }
}

impl Decoder for BytesCodec {
    type Item = Response;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.remaining();
        let bytes = src.copy_to_bytes(len);
        Ok(Some(Response::new(bytes)))
    }
}
