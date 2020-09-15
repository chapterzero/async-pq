use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;
use std::convert::TryInto;

type MResult<T> = Result<T, MessageDeserializerError>;

pub struct MessageDeserializer<'de> {
    input: &'de [u8],
}

impl<'de> MessageDeserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Self {
        MessageDeserializer { input }
    }

    fn parse_u32(&mut self) -> MResult<u32> {
        let b: [u8; 4] = self
            .input
            .try_into()
            .map_err(|e| MessageDeserializerError::InsufficientBytes(e))?;
        self.input = &self.input[4..];
        Ok(u32::from_be_bytes(b))
    }

    fn parse_char(&mut self) -> MResult<u8> {
        let res = self.input[0];
        self.input = &self.input[1..];
        Ok(res)
    }

    fn parse_string(&mut self) -> MResult<&'de str> {
        let mut iter = self.input.iter();
        match iter.position(|&x| x == 0) {
            Some(i) => {
                let s = std::str::from_utf8(&self.input[0..i])
                    .map_err(|e| MessageDeserializerError::Utf8Err(e))?;
                if self.input.len() >= i {
                    self.input = &self.input[i..];
                }
                Ok(s)
            }
            None => Err(MessageDeserializerError::NoNullBytes),
        }
    }
}

pub fn from_slice<'a, T>(s: &'a [u8]) -> MResult<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = MessageDeserializer::from_slice(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(MessageDeserializerError::TrailingBytes)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut MessageDeserializer<'de> {
    type Error = MessageDeserializerError;

    fn deserialize_u32<V>(self, visitor: V) -> MResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_u32()?)
    }
}

#[derive(Debug)]
pub enum MessageDeserializerError {
    Custom(String),
    TrailingBytes,
    InsufficientBytes(std::array::TryFromSliceError),
    NoNullBytes,
    Utf8Err(std::str::Utf8Error),
}

use std::fmt;
impl fmt::Display for MessageDeserializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when deserializing bytes into struct")
    }
}

impl std::error::Error for MessageDeserializerError {}

impl serde::de::Error for MessageDeserializerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        MessageDeserializerError::Custom(msg.to_string())
    }
}
