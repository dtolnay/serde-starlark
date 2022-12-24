mod error;
mod ser;

use crate::newline::WithNewline;
use crate::ser::{WriteMap, WriteSeq, WriteStarlark};
use serde::ser::{Impossible, Serialize};

pub struct Error {
    kind: crate::error::ErrorKind,
}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Serialize,
{
    value.serialize(Serializer)
}

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = WithNewline<WriteSeq<WriteStarlark>>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = WithNewline<WriteMap<WriteStarlark>>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_bool(v).map(newline)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i8(v).map(newline)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i16(v).map(newline)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i32(v).map(newline)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i64(v).map(newline)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_i128(v).map(newline)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u8(v).map(newline)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u16(v).map(newline)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u32(v).map(newline)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u64(v).map(newline)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_u128(v).map(newline)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_f32(v).map(newline)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_f64(v).map(newline)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_char(v).map(newline)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_str(v).map(newline)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_bytes(v).map(newline)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_none().map(newline)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new().serialize_some(value).map(newline)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new().serialize_unit().map(newline)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new()
            .serialize_unit_struct(name)
            .map(newline)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        WriteStarlark::new()
            .serialize_unit_variant(name, variant_index, variant)
            .map(newline)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new()
            .serialize_newtype_struct(name, value)
            .map(newline)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        WriteStarlark::new()
            .serialize_newtype_variant(name, variant_index, variant, value)
            .map(newline)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        WriteStarlark::new().serialize_seq(len).map(WithNewline)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        WriteStarlark::new().serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        WriteStarlark::new().serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        WriteStarlark::new().serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        WriteStarlark::new().serialize_map(len).map(WithNewline)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        WriteStarlark::new().serialize_struct(name, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        WriteStarlark::new().serialize_struct_variant(name, variant_index, variant, len)
    }
}

fn newline(mut starlark: String) -> String {
    starlark.push('\n');
    starlark
}

mod newline {
    use super::newline;
    use serde::ser::{Serialize, SerializeMap, SerializeSeq};

    pub struct WithNewline<S>(pub(crate) S);

    impl<S> SerializeSeq for WithNewline<S>
    where
        S: SerializeSeq<Ok = String>,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_element(value)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            self.0.end().map(newline)
        }
    }

    impl<S> SerializeMap for WithNewline<S>
    where
        S: SerializeMap<Ok = String>,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_key(key)
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize + ?Sized,
        {
            self.0.serialize_value(value)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            self.0.end().map(newline)
        }
    }
}
