use serde::de::{self, Visitor};
use serde::Deserialize;

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
    };
}

macro_rules! parse_int_fn {
    ($fn: ident, $t:ident, $fnb:ident) => {
        fn $fn(&mut self) -> MResult<$t> {
            let b = self.$fnb()?;
            Ok($t::from_be_bytes(b))
        }
    };
}

macro_rules! deserialize_int_fn {
    ($fn: ident, $vfn: ident, $pfn: ident) => {
        fn $fn<V>(self, visitor: V) -> MResult<V::Value>
        where
            V: Visitor<'de>,
        {
            visitor.$vfn(self.$pfn()?)
        }
    };
}

macro_rules! deserialize_unimplemented {
    ($fn: ident) => {
        fn $fn<V>(self, _visitor: V) -> MResult<V::Value>
        where
            V: Visitor<'de>,
        {
            unimplemented!()
        }
    };
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

    fn parse_str(&mut self) -> MResult<&'de str> {
        let mut iter = self.input.iter().skip(self.idx);
        match iter.position(|&x| x == b'\x00') {
            Some(i) => {
                let s = std::str::from_utf8(self.get_and_advance(i).unwrap())
                    .map_err(|e| MessageDeserializerError::Utf8Err(e))?;
                self.advance(1);
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
    if deserializer.idx == deserializer.input.len() {
        Ok(t)
    } else {
        Err(MessageDeserializerError::TrailingBytes)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut MessageDeserializer<'de> {
    type Error = MessageDeserializerError;

    fn deserialize_str<V>(self, visitor: V) -> MResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> MResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

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

    deserialize_unimplemented!(deserialize_any);
    deserialize_unimplemented!(deserialize_bool);
    deserialize_unimplemented!(deserialize_char);
    deserialize_unimplemented!(deserialize_bytes);
    deserialize_unimplemented!(deserialize_byte_buf);
    deserialize_unimplemented!(deserialize_option);
    deserialize_unimplemented!(deserialize_unit);
    deserialize_unimplemented!(deserialize_seq);
    deserialize_unimplemented!(deserialize_map);
    deserialize_unimplemented!(deserialize_identifier);
    deserialize_unimplemented!(deserialize_ignored_any);

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess::new(self))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> MResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(1, visitor)
    }
}

struct SeqAccess<'a, 'de> {
    de: &'a mut MessageDeserializer<'de>,
}

impl<'a, 'de> SeqAccess<'a, 'de> {
    fn new(de: &'a mut MessageDeserializer<'de>) -> Self {
        SeqAccess { de }
    }
}

impl<'a, 'de> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = MessageDeserializerError;

    fn next_element_seed<T>(&mut self, seed: T) -> MResult<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        Ok(Some(seed.deserialize(&mut *self.de).unwrap()))
    }
}

trait KnownByteLen {
    fn get_byte_len(&self) -> usize;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_struct() {
        #[derive(Deserialize, Debug)]
        struct Test {
            i: u32,
            j: i16,
        }
        let source: Vec<u8> = vec![0, 0, 0, 0x20, 0x1, 0x5E];
        let t: Test = from_slice(&source).unwrap();
        assert_eq!(32, t.i);
        assert_eq!(350, t.j);
    }

    #[test]
    fn test_deserialize_tuple() {
        #[derive(Deserialize, Debug)]
        struct Test {
            a: (u64, u32),
        }
        let source: Vec<u8> = vec![0, 0, 0, 0, 0, 0x1, 0xf4, 0, 0, 0, 0x2, 0x1];
        let t: Test = from_slice(&source).unwrap();
        assert_eq!(128000, t.a.0);
        assert_eq!(513, t.a.1);
    }

    #[test]
    fn test_deserialize_fixed_size() {
        #[derive(Deserialize, Debug)]
        struct Test {
            a: (u64, u32),
            b: [u8; 3],
        }
        let source: Vec<u8> = vec![0, 0, 0, 0, 0, 0x1, 0xf4, 0, 0, 0, 0x2, 0x1, 0x2, 0x1, 0xff];
        let t: Test = from_slice(&source).unwrap();
        assert_eq!(128000, t.a.0);
        assert_eq!(513, t.a.1);
        assert_eq!(&[0x2u8, 0x1, 0xff], &t.b);
    }

    #[test]
    fn test_deserialize_string() {
        #[derive(Deserialize, Debug)]
        struct Test {
            a: String,
            b: String,
        }
        let source: Vec<u8> = vec![
            0x53, 0x65, 0x72, 0x64, 0x65, 0, 0x52, 0x4F, 0x43, 0x4B, 0x53, 0x20, 0x21, 0,
        ];
        let t: Test = from_slice(&source).unwrap();
        assert_eq!("Serde", t.a);
        assert_eq!("ROCKS !", t.b);
    }

    #[test]
    fn test_deserialize_str() {
        #[derive(Deserialize, Debug)]
        struct Test<'a> {
            a: &'a str,
            b: &'a str,
        }
        let source: Vec<u8> = vec![
            0x53, 0x65, 0x72, 0x64, 0x65, 0, 0x52, 0x4F, 0x43, 0x4B, 0x53, 0x20, 0x21, 0,
        ];
        let t: Test = from_slice(&source).unwrap();
        assert_eq!("Serde", t.a);
        assert_eq!("ROCKS !", t.b);
    }
}
