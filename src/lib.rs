use serde::Serialize;

pub struct Error {}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ?Sized + Serialize,
{
    let _ = value;
    unimplemented!()
}
