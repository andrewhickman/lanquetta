use std::convert::TryFrom;
use std::fmt::Write;

use anyhow::Context;
use bytes::Buf;
use chrono::{SecondsFormat, TimeZone, Utc};
use prost::Message;
use prost_types::value::Kind;
use serde_json::{Number, Value};

use super::DecodeBuf;

pub(super) fn deserialize_timestamp<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let raw = prost_types::Timestamp::decode(buf)?;

    let dt = Utc
        .timestamp_opt(
            raw.seconds,
            u32::try_from(raw.nanos).context("invalid timestamp")?,
        )
        .single()
        .context("invalid timestamp")?;

    Ok(Value::from(dt.to_rfc3339_opts(SecondsFormat::AutoSi, true)))
}

pub(super) fn deserialize_duration<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let raw = prost_types::Duration::decode(buf)?;

    if raw.nanos == 0 {
        Ok(Value::from(format!("{}s", raw.seconds)))
    } else {
        Ok(Value::from(format!("{}.{:0>9}s", raw.seconds, raw.nanos)))
    }
}

pub(super) fn deserialize_struct<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let raw = prost_types::Struct::decode(buf)?;

    Ok(struct_to_json(raw))
}

pub(super) fn deserialize_double_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct DoubleValue {
        #[prost(double, tag = "1")]
        pub value: f64,
    }

    let raw = DoubleValue::decode(buf)?;
    Ok(super::double_to_json(raw.value))
}

pub(super) fn deserialize_float_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct FloatValue {
        #[prost(float, tag = "1")]
        pub value: f32,
    }

    let raw = FloatValue::decode(buf)?;

    Ok(super::float_to_json(raw.value))
}

pub(super) fn deserialize_int32_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct Int32Value {
        #[prost(int32, tag = "1")]
        pub value: i32,
    }

    let raw = Int32Value::decode(buf)?;

    Ok(Number::from(raw.value).into())
}

pub(super) fn deserialize_int64_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct Int64Value {
        #[prost(int64, tag = "1")]
        pub value: i64,
    }

    let raw = Int64Value::decode(buf)?;

    Ok(raw.value.to_string().into())
}

pub(super) fn deserialize_uint32_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(Message)]
    pub struct UInt32Value {
        #[prost(uint32, tag = "1")]
        pub value: u32,
    }

    let raw = UInt32Value::decode(buf)?;

    Ok(Number::from(raw.value).into())
}

pub(super) fn deserialize_uint64_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct UInt64Value {
        #[prost(uint64, tag = "1")]
        pub value: u64,
    }

    let raw = UInt64Value::decode(buf)?;

    Ok(raw.value.to_string().into())
}

pub(super) fn deserialize_string_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct StringValue {
        #[prost(string, tag = "1")]
        pub value: String,
    }

    let raw = StringValue::decode(buf)?;

    Ok(raw.value.into())
}

pub(super) fn deserialize_bytes_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct BytesValue {
        #[prost(bytes = "vec", tag = "1")]
        pub value: Vec<u8>,
    }

    let raw = BytesValue::decode(buf)?;

    Ok(base64::encode(raw.value).into())
}

pub(super) fn deserialize_bool_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(prost::Message)]
    pub struct BoolValue {
        #[prost(bool, tag = "1")]
        pub value: bool,
    }

    let raw = BoolValue::decode(buf)?;

    Ok(raw.value.into())
}

pub(super) fn deserialize_field_mask<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let raw = prost_types::FieldMask::decode(buf)?;

    let mut result = String::new();
    let mut iter = raw.paths.iter();
    if let Some(path) = iter.next() {
        path_to_camel_case(&mut result, path);
    }
    for path in iter {
        write!(result, ",").unwrap();
        path_to_camel_case(&mut result, path);
    }
    Ok(result.into())
}

fn path_to_camel_case(dst: &mut String, path: &str) {
    let mut parts = path.split(".");
    if let Some(part) = parts.next() {
        write!(dst, "{}", heck::AsLowerCamelCase(part)).unwrap();
    }
    for part in parts {
        write!(dst, ".{}", heck::AsLowerCamelCase(part)).unwrap();
    }
}

pub(super) fn deserialize_list_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let raw = prost_types::ListValue::decode(buf)?;

    Ok(list_to_json(raw))
}

pub(super) fn deserialize_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let raw = prost_types::Value::decode(buf)?;

    Ok(value_to_json(raw))
}

pub(super) fn deserialize_null_value<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    let _ = prost::encoding::decode_varint(buf)?;
    Ok(Value::Null)
}

pub(super) fn deserialize_empty<B>(buf: &mut DecodeBuf<B>) -> Result<Value, super::Error>
where
    B: Buf,
{
    #[derive(Message)]
    pub struct Empty {}

    let _ = Empty::decode(buf)?;

    Ok(Value::Object(Default::default()))
}

fn value_to_json(value: prost_types::Value) -> Value {
    match value.kind {
        Some(Kind::NullValue(_)) => Value::Null,
        Some(Kind::BoolValue(value)) => value.into(),
        Some(Kind::StringValue(value)) => value.into(),
        Some(Kind::NumberValue(value)) => super::double_to_json(value),
        Some(Kind::ListValue(value)) => list_to_json(value),
        Some(Kind::StructValue(value)) => struct_to_json(value),
        None => Value::Null,
    }
}

fn list_to_json(value: prost_types::ListValue) -> Value {
    Value::Array(value.values.into_iter().map(value_to_json).collect())
}

fn struct_to_json(value: prost_types::Struct) -> Value {
    Value::Object(
        value
            .fields
            .into_iter()
            .map(|(key, value)| (key, value_to_json(value)))
            .collect(),
    )
}
