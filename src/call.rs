use crate::FunctionCall;
use serde::ser::{
    Error, Impossible, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple,
    SerializeTupleStruct, Serializer,
};

impl<'a, A> Serialize for FunctionCall<'a, A>
where
    A: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.args.serialize(FunctionCallSerializer {
            function: self.function,
            delegate: serializer,
        })
    }
}

struct FunctionCallSerializer<'name, S> {
    function: &'name str,
    delegate: S,
}

impl<'name, S> FunctionCallSerializer<'name, S> {
    const UNSUPPORTED: &'static str = "unsupported function call argument type";
}

impl<'a, S> Serializer for FunctionCallSerializer<'a, S>
where
    S: Serializer,
{
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = FunctionCallArgs<S::SerializeTupleStruct>;
    type SerializeTuple = FunctionCallArgs<S::SerializeTupleStruct>;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = FunctionCallArgs<S::SerializeStruct>;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.unwrap_or(2);
        let mut delegate = self.delegate.serialize_tuple_struct("(", len)?;
        delegate.serialize_field(self.function)?;
        Ok(FunctionCallArgs { delegate })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let mut delegate = self.delegate.serialize_tuple_struct("(", len)?;
        delegate.serialize_field(self.function)?;
        Ok(FunctionCallArgs { delegate })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let mut delegate = self.delegate.serialize_tuple_struct("(", len)?;
        delegate.serialize_field(self.function)?;
        Ok(delegate)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len.unwrap_or(2);
        let mut delegate = self.delegate.serialize_struct("(", len)?;
        delegate.serialize_field("", self.function)?;
        Ok(FunctionCallArgs { delegate })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let mut delegate = self.delegate.serialize_struct("(", len)?;
        delegate.serialize_field("", self.function)?;
        Ok(delegate)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::custom(Self::UNSUPPORTED))
    }

    fn is_human_readable(&self) -> bool {
        self.delegate.is_human_readable()
    }
}

struct FunctionCallArgs<S> {
    delegate: S,
}

impl<S> SerializeSeq for FunctionCallArgs<S>
where
    S: SerializeTupleStruct,
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegate.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegate.end()
    }
}

impl<S> SerializeTuple for FunctionCallArgs<S>
where
    S: SerializeTupleStruct,
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegate.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegate.end()
    }
}

impl<S> SerializeMap for FunctionCallArgs<S>
where
    S: SerializeStruct,
{
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegate.serialize_field("*key", key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.delegate.serialize_field("*value", value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.delegate.end()
    }
}
