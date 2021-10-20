use std::fmt;

use bytes::{Buf, buf::Take};
use prost::encoding::WireType;
use serde::{Deserializer, de::{self, IntoDeserializer, Visitor, value::StrDeserializer}, forward_to_deserialize_any};

use crate::ty::MessageField;

use super::{Enum, Message, Scalar, Ty, TypeId, TypeMap};

pub struct Decoder<'a, B> {
    map: &'a TypeMap,
    ty: TypeId,
    buf: &'a mut B,
}

#[derive(Debug)]
pub struct Error {
    inner: anyhow::Error,
}

impl<'a, 'de, B> Decoder<'a, B>
where
    B: Buf,
{
    pub fn new(map: &'a TypeMap, ty: TypeId, buf: &'a mut B) -> Self {
        Decoder { map, ty, buf }
    }
}

impl<'a, 'de, B> Deserializer<'de> for Decoder<'a, B>
where
    B: Buf,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.map[self.ty] {
            Ty::Message(message) => deserialize_message(self.buf.take(self.buf.len()), self.map, WireType::LengthDelimited, message, visitor),
            _ => Err(de::Error::custom("expected top-level type to be a message")),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MessageDecoder<'a, B> {
    buf: Take<&'a mut B>,
    map: &'a TypeMap,
    wire_type: WireType,
    message: &'a Message,
}

fn deserialize_message<'de, B, V>(
    buf: Take<&mut B>,
    map: &TypeMap,
    wire_type: WireType,
    message: &Message,
    visitor: V,
) -> Result<V::Value, Error>
where
    B: Buf,
    V: Visitor<'de>,
{
    struct MapAccess<'a, B> {
        buf: Take<&'a mut B>,
        map: &'a TypeMap,
        message: &'a Message,
        current_key: Option<(WireType, &'a MessageField)>,
    }

    impl<'a, 'de, B> de::MapAccess<'de> for MapAccess<'a, B>
    where
        B: Buf,
    {
        type Error = Error;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: de::DeserializeSeed<'de>,
        {
            assert!(self.current_key.is_none());
            if self.buf.has_remaining() {
                let (tag, wire_type) = prost::encoding::decode_key(&mut self.buf)?;
                let field = &self.message.fields[tag as usize];
                self.current_key = Some((wire_type, field));

                let key_deserializer: StrDeserializer<'a, Error> =
                    field.json_name.as_str().into_deserializer();
                let key: K::Value = seed.deserialize(key_deserializer)?;
                Ok(Some(key))
            } else {
                Ok(None)
            }
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            let (wire_type, field) = self.current_key.expect("next_value called before next key");
            match &self.map[field.ty] {
                Ty::Message(_) => todo!(),
                Ty::Enum(_) => todo!(),
                Ty::Scalar(_) => todo!(),
                Ty::List(_) => todo!(),
                Ty::Map(_) => todo!(),
                Ty::Group(_) => todo!(),
            }
        }
    }

    if wire_type != WireType::LengthDelimited {
        return Err(de::Error::custom("invalid wire type for message"));
    }
    let len = prost::encoding::decode_varint(&mut buf)?;
    let limited_buf = buf.into_inner().take(len as usize);

    visitor.visit_map(MapAccess {
        buf: limited_buf,
        map,
        message,
        current_key: None,
    })
}

fn deserialize_enum<'de, B, V>(
    buf: Take<&mut B>,
    map: &TypeMap,
    enum_ty: &Enum,
    visitor: V,
) -> Result<V::Value, Error>
where
    B: Buf,
    V: Visitor<'de>,
{
    todo!()
}

fn deserialize_scalar<'de, B, V>(
    buf: Take<&mut B>,
    map: &TypeMap,
    scalar: &Scalar,
    visitor: V,
) -> Result<V::Value, Error>
where
    B: Buf,
    V: Visitor<'de>,
{
    todo!()
}

fn deserialize_list<'de, B, V>(
    buf: Take<&mut B>,
    map: &TypeMap,
    inner_ty: TypeId,
    visitor: V,
) -> Result<V::Value, Error>
where
    B: Buf,
    V: Visitor<'de>,
{
    todo!()
}

fn deserialize_map<'de, B, V>(
    buf: Take<&mut B>,
    map: &TypeMap,
    message_ty: TypeId,
    visitor: V,
) -> Result<V::Value, Error>
where
    B: Buf,
    V: Visitor<'de>,
{
    todo!()
}

fn deserialize_group<'de, B, V>(
    buf: Take<&mut B>,
    map: &TypeMap,
    message_ty: TypeId,
    visitor: V,
) -> Result<V::Value, Error>
where
    B: Buf,
    V: Visitor<'de>,
{
    todo!()
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn de::StdError + 'static)> {
        self.inner.source()
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error {
            inner: anyhow::Error::msg(msg.to_string()),
        }
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Self {
        Error { inner: err.into() }
    }
}
