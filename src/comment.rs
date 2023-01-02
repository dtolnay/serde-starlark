use crate::LineComment;
use serde::ser::{Serialize, SerializeTupleStruct, Serializer};

impl<'a, T> Serialize for LineComment<'a, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut comment = serializer.serialize_tuple_struct("#", 2)?;
        comment.serialize_field(&self.comment)?;
        comment.serialize_field(&self.value)?;
        comment.end()
    }
}
