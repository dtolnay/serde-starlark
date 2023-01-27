use crate::Assignment;
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};

impl<'a, T> Serialize for Assignment<'a, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut assignment = serializer.serialize_tuple_struct("=", 0)?;
        assignment.serialize_field(&self.identifier)?;
        assignment.serialize_field(&self.value)?;
        assignment.end()
    }
}
