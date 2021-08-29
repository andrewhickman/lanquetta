use std::{any::TypeId, fmt};

use serde::{
    de::{self, Visitor},
    forward_to_deserialize_any, Deserializer,
};

use super::TypeMap;

pub struct Decoder<'a> {
    map: &'a TypeMap,
    ty: TypeId,
}

#[derive(Debug)]
pub struct Error {
    inner: anyhow::Error,
}

impl<'a, 'de> Deserializer<'de> for Decoder<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
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
