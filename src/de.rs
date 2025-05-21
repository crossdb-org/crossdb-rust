use crate::{Row, Value};
use serde::de::{
    self, value::Error as DeError, DeserializeSeed, Deserializer, Error, IntoDeserializer,
    MapAccess, Visitor,
};

pub(crate) struct RowDeserializer<'de> {
    row: &'de Row<'de>,
    index: usize,
}

impl<'de> RowDeserializer<'de> {
    pub(crate) fn new(row: &'de Row<'de>) -> Self {
        Self { row, index: 0 }
    }
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
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string option
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

impl<'de> MapAccess<'de> for RowDeserializer<'de> {
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.index >= self.row.columns.len() {
            return Ok(None);
        }
        let name = self.row.columns.name(self.index);
        seed.deserialize(name.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if self.index >= self.row.values.len() {
            return Err(de::Error::custom("Value index out of bounds"));
        }
        let value = &self.row.values[self.index];
        let result = seed.deserialize(ValueDeserializer(value));
        self.index += 1;
        result
    }
}

struct ValueDeserializer<'a>(&'a Value<'a>);

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.0 {
            Value::Null => visitor.visit_none(),
            Value::I32(v) => visitor.visit_i32(v),
            Value::I64(v) => visitor.visit_i64(v),
            Value::U32(v) => visitor.visit_u32(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::F32(v) => visitor.visit_f32(v),
            Value::F64(v) => visitor.visit_f64(v),
            // TODO
            Value::Timestamp(v) => visitor.visit_i64(v),
            Value::String(v) => visitor.visit_str(v),
            // TODO: Deserialize Binary to Vec<u8>
            Value::Binary(_) => todo!(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Inet(_) => todo!(),
            Value::Mac(_) => todo!(),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.0 {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any struct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fmt::Debug;

    #[test]
    fn test_deserialize_value() {
        fn de<'a, T: Debug + PartialEq + Deserialize<'a>>(v: &'a Value<'a>, expected: T) {
            let v = ValueDeserializer(v);
            assert_eq!(Deserialize::deserialize(v), Ok(expected));
        }

        de(&Value::Null, ());
        de(&Value::Null, None::<i8>);
        de(&Value::Null, None::<String>);

        de(&Value::I32(1), 1_i32);
        de(&Value::I32(1), Some(1_i32));
        de(&Value::I32(1), Some(1_u128));

        de(&Value::I64(1), 1_i16);
        de(&Value::I64(1), Some(1_i16));
        de(&Value::I64(1), Some(1_u128));

        de(&Value::String("Hello"), String::from("Hello"));
        de(&Value::String("Hello"), Some(String::from("Hello")));
    }

    #[test]
    fn test_deserialize_row() {
        // TODO
    }
}
