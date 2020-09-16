use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;
use std::convert::TryInto;

type MResult<T> = Result<T, MessageDeserializerError>;

macro_rules! get_bytes_fn {
    ($fn:ident, $n:expr) => {
        fn $fn(&mut self) -> MResult<[u8; $n]> {
            self.get_and_advance($n)
                .map(|sl| {
                    let mut msg_len = [0u8; $n];
                    msg_len.copy_from_slice(sl);
                    msg_len
                })
            .ok_or(MessageDeserializerError::InsufficientBytes($n))
        }
    }
}

macro_rules! parse_int_fn {
    ($fn: ident, $t:ident, $fnb:ident) => {
        fn $fn(&mut self) -> MResult<$t> {
            let b = self.$fnb()?;
            Ok($t::from_be_bytes(b))
        }
    }
}


macro_rules! deserialize_int_fn {
    ($fn: ident, $vfn: ident, $pfn: ident) => {
        fn $fn<V>(self, visitor: V) -> MResult<V::Value>
        where
            V: Visitor<'de>,
        {
            visitor.$vfn(self.$pfn()?)
        }
    }
}

macro_rules! deserialize_unimplemented {
    ($fn: ident) => {
        fn $fn<V>(self, visitor: V) -> MResult<V::Value>
        where
            V: Visitor<'de>,
        {
            unimplemented!()
        }
    }
}

pub struct MessageDeserializer<'de> {
    input: &'de [u8],
    idx: usize,
}

impl<'de> MessageDeserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Self {
        MessageDeserializer { input, idx: 0 }
    }

    fn advance(&mut self, by: usize) {
        self.idx += by;
    }

    fn get_and_advance(&mut self, len: usize) -> Option<&'de [u8]> {
        if self.idx + len > self.input.len() {
            return None;
        }

        let res = &self.input[self.idx..self.idx + len];
        self.advance(len);
        Some(res)
    }

    // get bytes utility
    get_bytes_fn!(get_1_byte, 1);
    get_bytes_fn!(get_2_bytes, 2);
    get_bytes_fn!(get_4_bytes, 4);
    get_bytes_fn!(get_8_bytes, 8);

    // parsing from big endian bytes
    parse_int_fn!(parse_u8, u8, get_1_byte);
    parse_int_fn!(parse_u16, u16, get_2_bytes);
    parse_int_fn!(parse_u32, u32, get_4_bytes);
    parse_int_fn!(parse_u64, u64, get_8_bytes);
    parse_int_fn!(parse_i8, i8, get_1_byte);
    parse_int_fn!(parse_i16, i16, get_2_bytes);
    parse_int_fn!(parse_i32, i32, get_4_bytes);
    parse_int_fn!(parse_i64, i64, get_8_bytes);

    parse_int_fn!(parse_f32, f32, get_4_bytes);
    parse_int_fn!(parse_f64, f64, get_8_bytes);

    fn parse_char(&mut self) -> MResult<u8> {
        let b = self
            .get_and_advance(1)
            .ok_or(MessageDeserializerError::InsufficientBytes(1))?;
        Ok(b[0])
    }

    fn parse_str(&mut self) -> MResult<&'de str> {
        let mut iter = self.input.iter();
        match iter.position(|&x| x == b'\x00') {
            Some(i) => {
                let s = std::str::from_utf8(self.get_and_advance(i).unwrap())
                    .map_err(|e| MessageDeserializerError::Utf8Err(e))?;
                Ok(s)
            }
            None => Err(MessageDeserializerError::NoNullTerminator),
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

    deserialize_int_fn!(deserialize_u8, visit_u8, parse_u8);
    deserialize_int_fn!(deserialize_u16, visit_u16, parse_u16);
    deserialize_int_fn!(deserialize_u32, visit_u32, parse_u32);
    deserialize_int_fn!(deserialize_u64, visit_u64, parse_u64);
    deserialize_int_fn!(deserialize_i8, visit_i8, parse_i8);
    deserialize_int_fn!(deserialize_i16, visit_i16, parse_i16);
    deserialize_int_fn!(deserialize_i32, visit_i32, parse_i32);
    deserialize_int_fn!(deserialize_i64, visit_i64, parse_i64);
    deserialize_int_fn!(deserialize_f32, visit_f32, parse_f32);
    deserialize_int_fn!(deserialize_f64, visit_f64, parse_f64);

    fn deserialize_str<V>(self, visitor: V) -> MResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.parse_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> MResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    deserialize_unimplemented!(deserialize_any);
    deserialize_unimplemented!(deserialize_bool);
    deserialize_unimplemented!(deserialize_char);
    deserialize_unimplemented!(deserialize_bytes);
    deserialize_unimplemented!(deserialize_byte_buf);
    deserialize_unimplemented!(deserialize_option);
    deserialize_unimplemented!(deserialize_unit);
    deserialize_unimplemented!(deserialize_seq);

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
}

#[derive(Debug)]
pub enum MessageDeserializerError {
    Custom(String),
    TrailingBytes,
    InsufficientBytes(usize),
    NoNullTerminator,
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
