use std::fmt;

use anyhow::Context;
use bytes::Buf;
use prost::encoding::WireType;
use serde::{Deserializer, de::{self, Visitor}, forward_to_deserialize_any};
use serde_json::Value;

use super::{Enum, Message, Scalar, Ty, TypeId, TypeMap};

pub struct Decoder<'a, B> {
    map: &'a TypeMap,
    ty: TypeId,
    buf: DecodeBuf<'a, B>,
}

struct DecodeBuf<'a, B> {
    buf: &'a mut B,
    limit_stack: &'a mut Vec<usize>,
}

#[derive(Debug)]
pub struct Error {
    inner: anyhow::Error,
}

impl<'a, B> DecodeBuf<'a, B> {
    fn new(buf: &'a mut B, limit_stack: &'a mut Vec<usize>) -> Self {
        DecodeBuf {
            limit_stack,
            buf,
        }
    }

    fn reborrow<'b>(&'b mut self) -> DecodeBuf<'b, B> {
        DecodeBuf {
            buf: &mut *self.buf,
            limit_stack: &mut *self.limit_stack,
        }
    }

    fn push_limit(&mut self, limit: usize) {
        if let Some(limit)
        self.limit_stack.push(limit);
    }

    fn pop_limit(&mut self) {
        let limit = self.limit_stack.pop().expect("unbalanced stack")?;
        if let Some(prev_limit)
    }

    fn limit(&mut self)
}

impl<'a, B> Buf for DecodeBuf<'a, B> where B: Buf {
    fn remaining(&self) -> usize {
        self.buf.remaining().min(self.limit)
    }

    fn chunk(&self) -> &[u8] {
        &self.buf.chunk()[..self.remaining()]
    }

    fn advance(&mut self, cnt: usize) {
        self.buf.advance(cnt);
        self.limit = self.limit.checked_sub(cnt).expect("cnt > remaining");
    }
}

impl<'a, 'de, B> Decoder<'a, B>
where
    B: Buf,
{
    pub fn new(map: &'a TypeMap, ty: TypeId, buf: &'a mut B) -> Self {
        Decoder { map, ty, buf: DecodeBuf::new(buf) }
    }
}

impl<'a, 'de, B> Deserializer<'de> for Decoder<'a, B>
where
    B: Buf,
{
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut json = serde_json::Value::Object(Default::default());
        match &self.map[self.ty] {
            Ty::Message(message) => deserialize_message(
                self.buf.reborrow(),
                &mut json,
                self.map,
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

fn deserialize<B>(buf: DecodeBuf<B>, field_value: &mut Value, type_map: &TypeMap, wire_type: WireType, ty: TypeId) -> Result<(), Error> where B: Buf{
    match &type_map[ty] {
        Ty::Message(nested_message) => {
            deserialize_message_length_delimited(buf, field_value, type_map, wire_type, nested_message)?
        }
        Ty::Enum(enum_ty) => { *field_value = deserialize_enum(buf, type_map, enum_ty)? },
        Ty::Scalar(scalar) => { *field_value = deserialize_scalar(buf, type_map, *scalar)? },
        Ty::List(inner_ty) => deserialize_list(buf, field_value, wire_type, type_map, *inner_ty)?,
        Ty::Map(inner_ty) => deserialize_map(buf, field_value, wire_type, type_map, *inner_ty)?,
        Ty::Group(inner_ty) => deserialize_group(buf, field_value, type_map, *inner_ty)?,
    };
    Ok(())
}

fn deserialize_message<'de, B>(
    mut buf: DecodeBuf<'de, B>,
    value: &mut serde_json::Value,
    type_map: &TypeMap,
    message: &Message,
) -> Result<(), Error>
where
    B: Buf,
{
    let map = value.as_object_mut().expect("expected object type");

    while buf.has_remaining() {
        let (tag, wire_type) = prost::encoding::decode_key(&mut buf)?;
        let field = &message.fields[tag as usize];

        let key = field.json_name.clone();

        let default_value = type_map[field.ty].default_value();

        let field_value = map.entry(key).or_insert(default_value);

        deserialize(buf.reborrow(), field_value, type_map, wire_type, field.ty)?;
    }

    Ok(())
}

fn deserialize_message_length_delimited<'de, B>(
    mut buf: DecodeBuf<'de, B>,
    value: &mut serde_json::Value,
    type_map: &TypeMap,
    wire_type: WireType,
    message: &Message,
) -> Result<(), Error>
where
    B: Buf,
{
    if wire_type != WireType::LengthDelimited {
        return Err(de::Error::custom("invalid wire type for message"));
    }
    let len = prost::decode_length_delimiter(&mut buf)?;
    let buf = buf.with_limit(len);

    deserialize_message(buf, value, type_map, message)
}

fn deserialize_enum<'de, B>(
    mut buf: DecodeBuf<'de, B>,
    _: &TypeMap,
    enum_ty: &Enum,
) -> Result<Value, Error>
where
    B: Buf,
{
    let value = prost::encoding::decode_varint(&mut buf)?;
    if let Some(variant) = enum_ty.values.iter().find(|v| v.number as u64 == value) {
        Ok(Value::String(variant.name.to_owned()))
    } else {
        Ok(Value::Number(value.into()))
    }
}

fn deserialize_scalar<'de, B>(
    mut buf: DecodeBuf<'de, B>,
    _: &TypeMap,
    scalar: Scalar,
) -> Result<Value, Error>
where
    B: Buf,
{
    let ctx = prost::encoding::DecodeContext::default();
    match scalar {
        Scalar::Double => {
            let mut value: f64 = 0.0;
            prost::encoding::double::merge(WireType::SixtyFourBit, &mut value, &mut buf, ctx)?;
            match serde_json::Number::from_f64(value) {
                Some(number) => Ok(number.into()),
                None => {
                    if value == f64::INFINITY {
                        return Ok("Infinity".into())
                    } else if value == f64::NEG_INFINITY {
                        return Ok("-Infinity".into())
                    } else if value.is_nan() {
                        return Ok("NaN".into())
                    } else {
                        unreachable!("unexpected floating point value: {}", value)
                    }
                }
            }
        },
        Scalar::Float => {
            let mut value: f32 = 0.0;
            prost::encoding::float::merge(WireType::ThirtyTwoBit, &mut value, &mut buf, ctx)?;
            match serde_json::Number::from_f64(value.into()) {
                Some(number) => Ok(number.into()),
                None => {
                    if value == f32::INFINITY {
                        return Ok("Infinity".into())
                    } else if value == f32::NEG_INFINITY {
                        return Ok("-Infinity".into())
                    } else if value.is_nan() {
                        return Ok("NaN".into())
                    } else {
                        unreachable!("unexpected floating point value: {}", value)
                    }
                }
            }
        },
        Scalar::Int32 => {
            let mut value: i32 = 0;
            prost::encoding::int32::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Int64 => {
            let mut value: i64 = 0;
            prost::encoding::int64::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Uint32 => {
            let mut value: u32 = 0;
            prost::encoding::uint32::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Uint64 => {
            let mut value: u64 = 0;
            prost::encoding::uint64::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Sint32 => {
            let mut value: i32 = 0;
            prost::encoding::sint32::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Sint64 => {
            let mut value: i64 = 0;
            prost::encoding::sint64::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Fixed32 => {
            let mut value: u32 = 0;
            prost::encoding::fixed32::merge(WireType::ThirtyTwoBit, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Fixed64 => {
            let mut value: u64 = 0;
            prost::encoding::fixed64::merge(WireType::SixtyFourBit, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Sfixed32 => {
            let mut value: i32 = 0;
            prost::encoding::sfixed32::merge(WireType::ThirtyTwoBit, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Sfixed64 => {
            let mut value: i64 = 0;
            prost::encoding::sfixed64::merge(WireType::SixtyFourBit, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        },
        Scalar::Bool => {
            let mut value: bool = false;
            prost::encoding::bool::merge(WireType::Varint, &mut value, &mut buf, ctx)?;
            Ok(value.into())
        },
        Scalar::String => {
            let mut value: String = String::default();
            prost::encoding::string::merge(WireType::LengthDelimited, &mut value, &mut buf, ctx)?;
            Ok(value.into())
        },
        Scalar::Bytes => {
            let mut value: Vec<u8> = Vec::default();
            prost::encoding::bytes::merge(WireType::LengthDelimited, &mut value, &mut buf, ctx)?;
            Ok(serde_json::Value::String(base64::encode(value)))
        },
    }
}

fn deserialize_list<'de, B>(
    mut buf: DecodeBuf<'de, B>,
    value: &mut serde_json::Value,
    wire_type: WireType,
    type_map: &TypeMap,
    inner_ty: TypeId,
) -> Result<(), Error>
where
    B: Buf,
{
    let list = value.as_array_mut().expect("expected array type");

    if wire_type == WireType::LengthDelimited {
        // Packed
        let len = prost::decode_length_delimiter(&mut buf)?;
        let mut buf = buf.with_limit(len);
        while buf.has_remaining() {
            let mut value = type_map[inner_ty].default_value();
            deserialize(buf.reborrow(), &mut value, type_map, wire_type, inner_ty)?;
            list.push(value);
        }
    } else {
        // Unpacked
        let mut value = type_map[inner_ty].default_value();
        deserialize(buf.reborrow(), &mut value, type_map, wire_type, inner_ty)?;
        list.push(value);
    }

    Ok(())
}

fn deserialize_map<'de, B>(
    buf: DecodeBuf<'de, B>,
    value: &mut serde_json::Value,
    wire_type: WireType,
    type_map: &TypeMap,
    message_ty: TypeId,
) -> Result<(), Error>
where
    B: Buf,
{
    let mut list = serde_json::Value::Array(vec![]);
    deserialize_list(buf, &mut list, wire_type, type_map, message_ty)?;

    let map = value.as_object_mut().expect("expected map type");
    let list = list.as_array_mut().expect("expected array type");

    for mut entry in list.drain(..) {
        let key = match entry.as_object_mut().context("invalid type for map entry")?.remove("key") {
            Some(Value::String(string)) => string,
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Number(number)) => number.to_string(),
            Some(_) => return Err(anyhow::format_err!("invalid type for map entry").into()),
            None => return Err(anyhow::format_err!("no key found for map entry").into()),
        };
        let value = match entry.as_object_mut().context("invalid type for map entry")?.remove("value") {
            Some(value) => value,
            None => return Err(anyhow::format_err!("no key found for map entry").into()),
        };

        map.insert(key, value);
    }

    Ok(())
}

fn deserialize_group<'de, B>(
    _buf: DecodeBuf<'de, B>,
    _value: &mut serde_json::Value,
    _map: &TypeMap,
    _message_ty: TypeId,
) -> Result<(), Error>
where
    B: Buf,
{
    unimplemented!()
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

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error { inner: err.into() }
    }
}
