use crate::error;
use crate::Error;
use serde::ser::{Impossible, Serialize, SerializeMap, SerializeSeq};
use std::fmt::Write;
use std::iter;

pub struct WriteStarlark {
    output: String,
    indent: usize,
}

impl WriteStarlark {
    pub(crate) fn new() -> Serializer<Self> {
        Serializer {
            write: WriteStarlark {
                output: String::new(),
                indent: 0,
            },
        }
    }

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

pub trait MutableWriteStarlark {
    type Ok;
    fn mutable(&mut self) -> &mut WriteStarlark;
    fn output(self) -> Self::Ok;
}

impl MutableWriteStarlark for WriteStarlark {
    type Ok = String;
    fn mutable(&mut self) -> &mut WriteStarlark {
        self
    }
    fn output(self) -> Self::Ok {
        self.output
    }
}

impl MutableWriteStarlark for &mut WriteStarlark {
    type Ok = ();
    fn mutable(&mut self) -> &mut WriteStarlark {
        self
    }
    fn output(self) -> Self::Ok {}
}

pub(crate) struct Serializer<W> {
    write: W,
}

impl<W> serde::Serializer for Serializer<W>
where
    W: MutableWriteStarlark,
{
    type Ok = W::Ok;
    type Error = Error;
    type SerializeSeq = WriteSeq<W>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = WriteMap<W>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        let write = self.write.mutable();
        write.output.push_str(if v { "True" } else { "False" });
        Ok(self.write.output())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i32(i32::from(v))
    }

    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        let write = self.write.mutable();
        write!(write.output, "{}", v).unwrap();
        Ok(self.write.output())
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

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        let write = self.write.mutable();

        // Reference:
        // https://github.com/bazelbuild/starlark/blob/master/spec.md#string-literals
        write.output.reserve(v.len() + 2);
        write.output.push('"');
        let mut chars = v.chars().peekable();
        while let Some(ch) = chars.next() {
            if let Some(escape) = match ch {
                '\x07' => Some('a'), // alert or bell
                '\x08' => Some('b'), // backspace
                '\x0C' => Some('f'), // form feed
                '\n' => Some('n'),   // line feed
                '\r' => Some('r'),   // carriage return
                '\t' => Some('t'),   // horizontal tab
                '\x0B' => Some('v'), // vertical tab
                '"' => Some('"'),
                '\\' => Some('\\'),
                _ => None,
            } {
                write.output.push('\\');
                write.output.push(escape);
            } else if ch.is_ascii_control()
                && (ch as u8 >= 0o100 || chars.peek().map_or(true, |next| !next.is_digit(8)))
            {
                // Starlark has variable-width octal escapes: \0 through \177.
                // In order to use it we need to make sure the next character is
                // not going to be an octal digit.
                write!(write.output, "\\{:o}", ch as u8).unwrap();
            } else if ch.is_control() {
                if ch <= '\x7F' {
                    write!(write.output, "\\x{:02X}", ch as u8).unwrap();
                } else if ch <= '\u{FFFF}' {
                    write!(write.output, "\\u{:04X}", ch as u16).unwrap();
                } else {
                    write!(write.output, "\\U{:08X}", ch as u32).unwrap();
                }
            } else {
                write.output.push(ch);
            }
        }
        write.output.push('"');

        Ok(self.write.output())
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

    fn serialize_unit(mut self) -> Result<Self::Ok, Self::Error> {
        let write = self.write.mutable();
        write.output.push_str("None");
        Ok(self.write.output())
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

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let newlines = len.map_or(true, |len| len > 1);
        let write = self.write.mutable();
        write.output.push('[');
        if newlines {
            write.indent();
        }
        Ok(WriteSeq {
            write: self.write,
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

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let newlines = len.map_or(true, |len| len > 0);
        let write = self.write.mutable();
        write.output.push('{');
        if newlines {
            write.indent();
        }
        Ok(WriteMap {
            write: self.write,
            newlines,
        })
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

pub struct WriteSeq<W> {
    write: W,
    newlines: bool,
}

impl<W> SerializeSeq for WriteSeq<W>
where
    W: MutableWriteStarlark,
{
    type Ok = W::Ok;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let write = self.write.mutable();
        if self.newlines {
            write.newline();
        }
        value.serialize(Serializer { write: &mut *write })?;
        if self.newlines {
            write.output.push(',');
        }
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let write = self.write.mutable();
        if self.newlines {
            write.unindent();
        }
        write.output.push(']');
        Ok(self.write.output())
    }
}

pub struct WriteMap<W> {
    write: W,
    newlines: bool,
}

impl<W> SerializeMap for WriteMap<W>
where
    W: MutableWriteStarlark,
{
    type Ok = W::Ok;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let write = self.write.mutable();
        write.newline();
        key.serialize(Serializer { write: &mut *write })?;
        write.output.push_str(": ");
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let write = self.write.mutable();
        value.serialize(Serializer { write: &mut *write })?;
        write.output.push(',');
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let write = self.write.mutable();
        if self.newlines {
            write.unindent();
        }
        write.output.push('}');
        Ok(self.write.output())
    }
}
