mod well_known;

use std::fmt;

use anyhow::Context;
use bytes::Buf;
use prost::encoding::{DecodeContext, WireType};
use serde::{
    de::{self, Visitor},
    forward_to_deserialize_any, Deserializer,
};
use serde_json::Value;

use super::{Enum, Message, Scalar, Ty, TypeId, TypeMap};

pub struct Decoder<'a, B> {
    map: &'a TypeMap,
    ty: TypeId,
    buf: DecodeBuf<B>,
}

struct DecodeBuf<B> {
    buf: B,
    limit_stack: Vec<usize>,
}

#[derive(Debug)]
pub struct Error {
    inner: anyhow::Error,
}

impl<B> DecodeBuf<B> {
    fn new(buf: B) -> Self {
        DecodeBuf {
            limit_stack: Vec::new(),
            buf,
        }
    }

    fn push_limit(&mut self, limit: usize) -> Result<(), Error> {
        if let Some(prev_limit) = self.limit_stack.last_mut() {
            *prev_limit = prev_limit
                .checked_sub(limit)
                .ok_or_else(|| Error::from(anyhow::format_err!("limit too large")))?;
        }
        self.limit_stack.push(limit);
        Ok(())
    }

    fn pop_limit(&mut self) {
        let prev_limit = self.limit_stack.pop();
        debug_assert_eq!(prev_limit, Some(0));
    }
}

impl<B> Buf for DecodeBuf<B>
where
    B: Buf,
{
    fn remaining(&self) -> usize {
        if let Some(&limit) = self.limit_stack.last() {
            self.buf.remaining().min(limit)
        } else {
            self.buf.remaining()
        }
    }

    fn chunk(&self) -> &[u8] {
        &self.buf.chunk()[..self.remaining()]
    }

    fn advance(&mut self, cnt: usize) {
        self.buf.advance(cnt);
        if let Some(limit) = self.limit_stack.last_mut() {
            *limit = limit.checked_sub(cnt).expect("cnt > remaining");
        }
    }
}

impl<'a, B> Decoder<'a, B>
where
    B: Buf,
{
    pub(crate) fn new(map: &'a TypeMap, ty: TypeId, buf: B) -> Self {
        Decoder {
            map,
            ty,
            buf: DecodeBuf::new(buf),
        }
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
            Ty::Message(message) => {
                deserialize_message(&mut self.buf, &mut json, self.map, message)
            }
            _ => Err(de::Error::custom("expected top-level type to be a message")),
        }?;

        json.deserialize_any(visitor)
            .map_err(|e| Error { inner: e.into() })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

fn deserialize<B>(
    buf: &mut DecodeBuf<B>,
    field_value: &mut Value,
    type_map: &TypeMap,
    wire_type: WireType,
    ty: TypeId,
) -> Result<(), Error>
where
    B: Buf,
{
    match &type_map[ty] {
        Ty::Message(nested_message) => deserialize_message_length_delimited(
            buf,
            field_value,
            type_map,
            wire_type,
            nested_message,
        )?,
        Ty::Enum(enum_ty) => *field_value = deserialize_enum(buf, type_map, enum_ty)?,
        Ty::Scalar(scalar) => *field_value = deserialize_scalar(buf, type_map, *scalar)?,
        Ty::List(inner_ty) => deserialize_list(buf, field_value, wire_type, type_map, *inner_ty)?,
        Ty::Map(inner_ty) => deserialize_map(buf, field_value, wire_type, type_map, *inner_ty)?,
        Ty::Group(inner_ty) => deserialize_group(buf, field_value, type_map, *inner_ty)?,
    };

    Ok(())
}

fn deserialize_message<B>(
    mut buf: &mut DecodeBuf<B>,
    value: &mut serde_json::Value,
    type_map: &TypeMap,
    message: &Message,
) -> Result<(), Error>
where
    B: Buf,
{
    // Check for well-known types with special JSON mappings
    match &*message.name {
        ".google.protobuf.Timestamp" => {
            *value = well_known::deserialize_timestamp(buf)?;
            return Ok(());
        }
        ".google.protobuf.Duration" => {
            *value = well_known::deserialize_duration(buf)?;
            return Ok(());
        }
        ".google.protobuf.Struct" => {
            *value = well_known::deserialize_struct(buf)?;
            return Ok(());
        }
        ".google.protobuf.FloatValue" => {
            *value = well_known::deserialize_float_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.DoubleValue" => {
            *value = well_known::deserialize_double_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.Int32Value" => {
            *value = well_known::deserialize_int32_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.Int64Value" => {
            *value = well_known::deserialize_int64_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.UInt32Value" => {
            *value = well_known::deserialize_uint32_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.UInt64Value" => {
            *value = well_known::deserialize_uint64_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.BoolValue" => {
            *value = well_known::deserialize_bool_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.StringValue" => {
            *value = well_known::deserialize_string_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.BytesValue" => {
            *value = well_known::deserialize_bytes_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.FieldMask" => {
            *value = well_known::deserialize_field_mask(buf)?;
            return Ok(());
        }
        ".google.protobuf.ListValue" => {
            *value = well_known::deserialize_list_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.Value" => {
            *value = well_known::deserialize_value(buf)?;
            return Ok(());
        }
        ".google.protobuf.Empty" => {
            *value = well_known::deserialize_empty(buf)?;
            return Ok(());
        }
        _ => (),
    }

    let map = value.as_object_mut().expect("expected object type");

    while buf.has_remaining() {
        let (tag, wire_type) = prost::encoding::decode_key(&mut buf)?;
        if let Some(field) = message.fields.get(tag as usize) {
            let key = field.json_name.clone();

            let field_value = map
                .entry(key)
                .or_insert_with(|| type_map[field.ty].default_value());

            deserialize(buf, field_value, type_map, wire_type, field.ty)?;
        } else {
            // Skip field
            prost::encoding::skip_field(wire_type, tag, &mut buf, DecodeContext::default())?;
        }
    }

    Ok(())
}

fn deserialize_message_length_delimited<'de, B>(
    mut buf: &mut DecodeBuf<B>,
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
    buf.push_limit(len)?;

    deserialize_message(buf, value, type_map, message)?;

    buf.pop_limit();
    Ok(())
}

fn deserialize_enum<B>(buf: &mut DecodeBuf<B>, _: &TypeMap, enum_ty: &Enum) -> Result<Value, Error>
where
    B: Buf,
{
    // Check for well-known types with special JSON mappings
    match &*enum_ty.name {
        ".google.protobuf.NullValue" => return well_known::deserialize_null_value(buf),
        _ => (),
    }

    let value = prost::encoding::decode_varint(buf)?;
    if let Some(variant) = enum_ty.values.iter().find(|v| v.number as u64 == value) {
        Ok(Value::String(variant.name.to_owned()))
    } else {
        Ok(Value::Number(value.into()))
    }
}

fn deserialize_scalar<B>(
    buf: &mut DecodeBuf<B>,
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
            prost::encoding::double::merge(WireType::SixtyFourBit, &mut value, buf, ctx)?;
            Ok(double_to_json(value))
        }
        Scalar::Float => {
            let mut value: f32 = 0.0;
            prost::encoding::float::merge(WireType::ThirtyTwoBit, &mut value, buf, ctx)?;
            Ok(float_to_json(value))
        }
        Scalar::Int32 => {
            let mut value: i32 = 0;
            prost::encoding::int32::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        }
        Scalar::Int64 => {
            let mut value: i64 = 0;
            prost::encoding::int64::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(serde_json::Value::from(value.to_string()))
        }
        Scalar::Uint32 => {
            let mut value: u32 = 0;
            prost::encoding::uint32::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        }
        Scalar::Uint64 => {
            let mut value: u64 = 0;
            prost::encoding::uint64::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(serde_json::Value::from(value.to_string()))
        }
        Scalar::Sint32 => {
            let mut value: i32 = 0;
            prost::encoding::sint32::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        }
        Scalar::Sint64 => {
            let mut value: i64 = 0;
            prost::encoding::sint64::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(serde_json::Value::from(value.to_string()))
        }
        Scalar::Fixed32 => {
            let mut value: u32 = 0;
            prost::encoding::fixed32::merge(WireType::ThirtyTwoBit, &mut value, buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        }
        Scalar::Fixed64 => {
            let mut value: u64 = 0;
            prost::encoding::fixed64::merge(WireType::SixtyFourBit, &mut value, buf, ctx)?;
            Ok(serde_json::Value::from(value.to_string()))
        }
        Scalar::Sfixed32 => {
            let mut value: i32 = 0;
            prost::encoding::sfixed32::merge(WireType::ThirtyTwoBit, &mut value, buf, ctx)?;
            Ok(serde_json::Number::from(value).into())
        }
        Scalar::Sfixed64 => {
            let mut value: i64 = 0;
            prost::encoding::sfixed64::merge(WireType::SixtyFourBit, &mut value, buf, ctx)?;
            Ok(serde_json::Value::from(value.to_string()))
        }
        Scalar::Bool => {
            let mut value: bool = false;
            prost::encoding::bool::merge(WireType::Varint, &mut value, buf, ctx)?;
            Ok(value.into())
        }
        Scalar::String => {
            let mut value: String = String::default();
            prost::encoding::string::merge(WireType::LengthDelimited, &mut value, buf, ctx)?;
            Ok(value.into())
        }
        Scalar::Bytes => {
            let mut value: Vec<u8> = Vec::default();
            prost::encoding::bytes::merge(WireType::LengthDelimited, &mut value, buf, ctx)?;
            Ok(serde_json::Value::String(base64::encode(value)))
        }
    }
}

fn double_to_json(value: f64) -> Value {
    match serde_json::Number::from_f64(value) {
        Some(number) => number.into(),
        None => {
            if value == f64::INFINITY {
                "Infinity".into()
            } else if value == f64::NEG_INFINITY {
                "-Infinity".into()
            } else if value.is_nan() {
                "NaN".into()
            } else {
                unreachable!("unexpected floating point value: {}", value)
            }
        }
    }
}

fn float_to_json(value: f32) -> Value {
    match serde_json::Number::from_f64(value.into()) {
        Some(number) => number.into(),
        None => {
            if value == f32::INFINITY {
                "Infinity".into()
            } else if value == f32::NEG_INFINITY {
                "-Infinity".into()
            } else if value.is_nan() {
                "NaN".into()
            } else {
                unreachable!("unexpected floating point value: {}", value)
            }
        }
    }
}

fn deserialize_list<'de, B>(
    buf: &mut DecodeBuf<B>,
    value: &mut serde_json::Value,
    wire_type: WireType,
    type_map: &TypeMap,
    inner_ty: TypeId,
) -> Result<(), Error>
where
    B: Buf,
{
    let list = value.as_array_mut().expect("expected array type");

    let ty = &type_map[inner_ty];

    if wire_type == WireType::LengthDelimited && ty.is_numeric() {
        // Packed
        let len = prost::decode_length_delimiter(&mut *buf)?;
        buf.push_limit(len)?;
        while buf.has_remaining() {
            let mut value = ty.default_value();
            deserialize(buf, &mut value, type_map, wire_type, inner_ty)?;
            list.push(value);
        }
        buf.pop_limit();
    } else {
        // Unpacked
        let mut value = type_map[inner_ty].default_value();
        deserialize(buf, &mut value, type_map, wire_type, inner_ty)?;
        list.push(value);
    }

    Ok(())
}

fn deserialize_map<B>(
    buf: &mut DecodeBuf<B>,
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
        let key = match entry
            .as_object_mut()
            .context("invalid type for map entry")?
            .remove("key")
        {
            Some(Value::String(string)) => string,
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Number(number)) => number.to_string(),
            Some(_) => return Err(anyhow::format_err!("invalid type for map entry").into()),
            None => return Err(anyhow::format_err!("no key found for map entry").into()),
        };
        let value = match entry
            .as_object_mut()
            .context("invalid type for map entry")?
            .remove("value")
        {
            Some(value) => value,
            None => return Err(anyhow::format_err!("no key found for map entry").into()),
        };

        map.insert(key, value);
    }

    Ok(())
}

fn deserialize_group<B>(
    _buf: &mut DecodeBuf<B>,
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
