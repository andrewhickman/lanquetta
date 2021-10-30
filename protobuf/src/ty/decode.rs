use std::fmt;

use bytes::{buf::Take, Buf};
use prost::encoding::WireType;
use serde::{
    de::{self, Visitor},
    forward_to_deserialize_any, Deserializer,
};
use serde_json::{Map, Value};

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
        let json: Value = match &self.map[self.ty] {
            Ty::Message(message) => deserialize_message(
                self.buf,
                self.map,
                WireType::LengthDelimited,
                message,
            ),
            _ => Err(de::Error::custom("expected top-level type to be a message")),
        }?;

        json.deserialize_any(visitor).map_err(|e| Error { inner: e.into() })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MessageDecoder<'a, B> {
    buf: Take<&'a mut B>,
    type_map: &'a TypeMap,
    wire_type: WireType,
    message: &'a Message,
}

fn deserialize_message<'de, B>(
    buf: &mut B,
    type_map: &TypeMap,
    wire_type: WireType,
    message: &Message,
) -> Result<Value, Error>
where
    B: Buf,
{
    if wire_type != WireType::LengthDelimited {
        return Err(de::Error::custom("invalid wire type for message"));
    }
    let len = prost::encoding::decode_varint(buf)?;
    let mut buf = buf.take(len as usize);

    let mut map = Map::new();

    while buf.has_remaining() {
        let (tag, wire_type) = prost::encoding::decode_key(&mut buf)?;
        let field = &message.fields[tag as usize];

        let key = field.json_name.to_owned();

        let value = match &type_map[field.ty] {
            Ty::Message(nested_message) => {
                deserialize_message(&mut buf, type_map, wire_type, nested_message)
            }
            Ty::Enum(enum_ty) => deserialize_enum(&mut buf, type_map, enum_ty),
            Ty::Scalar(scalar) => deserialize_scalar(&mut buf, type_map, *scalar),
            Ty::List(inner_ty) => deserialize_list(&mut buf, type_map, *inner_ty),
            Ty::Map(inner_ty) => deserialize_map(&mut buf, type_map, *inner_ty),
            Ty::Group(_) => todo!(),
        }?;

        map.insert(key, value);
    }

    Ok(Value::Object(map))
}

fn deserialize_enum<'de, B>(
    buf: &mut B,
    map: &TypeMap,
    enum_ty: &Enum,
) -> Result<Value, Error>
where
    B: Buf,
{
    let value = prost::encoding::decode_varint(buf)?;
    if let Some(variant) = enum_ty.values.iter().find(|v| v.number as u64 == value) {
        Ok(Value::String(variant.name.to_owned()))
    } else {
        Ok(Value::Number(value.into()))
    }
}

fn deserialize_scalar<'de, B>(
    buf: &mut B,
    map: &TypeMap,
    scalar: Scalar,
) -> Result<Value, Error>
where
    B: Buf,
{
    match scalar {
        Scalar::Double => todo!(),
        Scalar::Float => todo!(),
        Scalar::Int64 => todo!(),
        Scalar::Uint64 => todo!(),
        Scalar::Int32 => todo!(),
        Scalar::Fixed64 => todo!(),
        Scalar::Fixed32 => todo!(),
        Scalar::Bool => todo!(),
        Scalar::String => todo!(),
        Scalar::Bytes => todo!(),
        Scalar::Uint32 => todo!(),
        Scalar::Sfixed32 => todo!(),
        Scalar::Sfixed64 => todo!(),
        Scalar::Sint32 => todo!(),
        Scalar::Sint64 => todo!(),
    }
}

fn deserialize_list<'de, B>(
    buf: &mut B,
    map: &TypeMap,
    inner_ty: TypeId,
) -> Result<Value, Error>
where
    B: Buf,
{
    todo!()
}

fn deserialize_map<'de, B>(
    buf: &mut B,
    map: &TypeMap,
    message_ty: TypeId,
) -> Result<Value, Error>
where
    B: Buf,
{
    todo!()
}

fn deserialize_group<'de, B>(
    buf: &mut B,
    map: &TypeMap,
    message_ty: TypeId,
) -> Result<Value, Error>
where
    B: Buf,
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
