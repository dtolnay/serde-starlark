use crate::error;
use crate::Error;
use serde::ser::{Impossible, Serialize, SerializeSeq, Serializer};
use std::fmt::Write;
use std::iter;

pub(crate) struct WriteStarlark {
    output: String,
    indent: usize,
}

impl WriteStarlark {
    fn newline(&mut self) {
        let indent = iter::repeat(' ').take(self.indent);
        self.output.extend(iter::once('\n').chain(indent));
    }

    fn indent(&mut self) {
        self.indent += 4;
    }

    fn unindent(&mut self) {
        self.indent -= 4;
        self.newline();
    }
}

impl<'a> Serializer for &'a mut WriteStarlark {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = WriteSeq<'a>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(if v { "True" } else { "False" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        write!(self.output, "{}", v).unwrap();
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        match i32::try_from(v) {
            Ok(v) => self.serialize_i32(v),
            Err(_) => Err(error::unsupported_i64(v)),
        }
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        match i32::try_from(v) {
            Ok(v) => self.serialize_i32(v),
            Err(_) => Err(error::unsupported_i128(v)),
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        match i32::try_from(v) {
            Ok(v) => self.serialize_i32(v),
            Err(_) => Err(error::unsupported_u32(v)),
        }
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        match i32::try_from(v) {
            Ok(v) => self.serialize_i32(v),
            Err(_) => Err(error::unsupported_u64(v)),
        }
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        match i32::try_from(v) {
            Ok(v) => self.serialize_i32(v),
            Err(_) => Err(error::unsupported_u128(v)),
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(error::unsupported_f32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(error::unsupported_f64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Err(error::unsupported_char(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(error::unsupported_bytes())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output.push_str("None");
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(error::unsupported_unit_struct(name))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(error::unsupported_enum(name, variant))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let newlines = len.map_or(true, |len| len > 1);
        self.output.push('[');
        if newlines {
            self.indent();
        }
        Ok(WriteSeq {
            serializer: self,
            newlines,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(error::unsupported_tuple())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(error::unsupported_enum(name, variant))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unimplemented!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(error::unsupported_enum(name, variant))
    }
}

pub(crate) struct WriteSeq<'a> {
    serializer: &'a mut WriteStarlark,
    newlines: bool,
}

impl SerializeSeq for WriteSeq<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        if self.newlines {
            self.serializer.newline();
        }
        value.serialize(&mut *self.serializer)?;
        if self.newlines {
            self.serializer.output.push(',');
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.newlines {
            self.serializer.unindent();
        }
        self.serializer.output.push(']');
        Ok(())
    }
}
