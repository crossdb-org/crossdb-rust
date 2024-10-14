use crate::Row;
use serde::de::{value::Error as DeError, Error, Visitor};
use serde::Deserializer;

pub(crate) struct RowDeserializer<'de> {
    pub(crate) row: &'de Row<'de>,
}

impl<'de> Deserializer<'de> for RowDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::custom("Expects a struct"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}
