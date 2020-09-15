use serde::Serialize;

type MResult<T> = Result<T, MessageSerializerError>;

pub struct MessageSerializer {
    output: Vec<u8>,
}

pub fn to_message<T: Serialize>(value: &T) -> Result<Vec<u8>, MessageSerializerError> {
    let mut serializer = MessageSerializer { output: vec![] };
    value.serialize(&mut serializer)?;
    // add nul terminator at the end
    serializer.output.push(0);
    Ok(serializer.output)
}

// count total of message length (including the length u32 bytes)
// and return serialized message with len included at len_pos index
pub fn to_message_with_len<T: Serialize>(value: &T, len_pos: usize) -> Result<Vec<u8>, MessageSerializerError> {
    let mut output = to_message(value)?;
    let len = (output.len() as u32 + 4).to_be_bytes();
    let mut v = output.split_off(len_pos);
    output.extend(&len);
    output.append(&mut v);
    Ok(output)
}

impl MessageSerializer {
    fn generic_bytes<T: AsRef<[u8]>>(&mut self, v: T) -> MResult<()> {
        self.output.extend(v.as_ref());
        Ok(())
    }
}

impl<'a> serde::Serializer for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> MResult<()> {
        let b = if v { 49u8 } else { 48u8 };
        Ok(self.output.push(b))
    }

    fn serialize_i8(self, v: i8) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_i16(self, v: i16) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_i32(self, v: i32) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_i64(self, v: i64) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_u8(self, v: u8) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_u16(self, v: u16) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_u32(self, v: u32) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_u64(self, v: u64) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_f32(self, v: f32) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_f64(self, v: f64) -> MResult<()> {
        self.generic_bytes(v.to_be_bytes())
    }

    fn serialize_char(self, v: char) -> MResult<()> {
        let mut b = [0; 4];
        v.encode_utf8(&mut b);
        Ok(self.output.extend(&b))
    }

    fn serialize_str(self, v: &str) -> MResult<()> {
        self.generic_bytes(v)?;
        Ok(self.output.push(0))
    }

    fn serialize_bytes(self, v: &[u8]) -> MResult<()> {
        self.generic_bytes(v)
    }

    fn serialize_none(self) -> MResult<()> {
        self.serialize_unit()
    }

    fn serialize_seq(self, _len: Option<usize>) -> MResult<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_some<T>(self, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> MResult<()> {
        Ok(self.output.push(0))
    }

    fn serialize_map(self, _len: Option<usize>) -> MResult<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> MResult<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> MResult<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> MResult<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(value.serialize(&mut *self)?)
    }

    fn serialize_tuple(self, len: usize) -> MResult<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> MResult<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> MResult<Self::SerializeTupleVariant> {
        Err(MessageSerializerError::TuppleVariantNotImplemented)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> MResult<Self::SerializeStructVariant> {
        Err(MessageSerializerError::StructVariantNotImplemented)
    }
}

impl<'a> serde::ser::SerializeSeq for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTuple for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_key<T>(&mut self, key: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        // only serialize the value
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut MessageSerializer {
    type Ok = ();
    type Error = MessageSerializerError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> MResult<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> MResult<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum MessageSerializerError {
    Custom(String),
    TuppleVariantNotImplemented,
    StructVariantNotImplemented,
}

use std::fmt;
impl fmt::Display for MessageSerializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when serializing struct into bytes")
    }
}

impl std::error::Error for MessageSerializerError {}

impl serde::ser::Error for MessageSerializerError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        MessageSerializerError::Custom(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::auth::StartupMessage;

    #[test]
    fn test_serialize_startup_message() {
        let m = StartupMessage::new("abcde", Some("mydb"));
        let bytes = to_message_with_len(&m, 0).unwrap();
        let expected = vec![
            0, 0, 0, 34, 0, 3, 0, 0, 117, 115, 101, 114, 0, 97, 98, 99, 100, 101, 0, 100, 97, 116,
            97, 98, 97, 115, 101, 0, 109, 121, 100, 98, 0, 0,
        ];
        assert_eq!(34, bytes.len());
        assert_eq!(expected, bytes);
    }

    #[test]
    pub fn test_startup_message_no_db() {
        let m = StartupMessage::new("abcde", None);
        let bytes = to_message_with_len(&m, 0).unwrap();
        let expected = vec![
            0, 0, 0, 20, 0, 3, 0, 0, 117, 115, 101, 114, 0, 97, 98, 99, 100, 101, 0, 0,
        ];
        assert_eq!(20, bytes.len());
        assert_eq!(expected, bytes);
    }
}
