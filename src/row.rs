use crate::{de::RowDeserializer, Columns, Result, Value};
use serde::de::{value::Error as DeError, DeserializeOwned};

#[derive(Debug)]
pub struct Row<'a> {
    pub(crate) columns: Columns,
    pub(crate) values: Vec<Value<'a>>,
}

impl Row<'_> {
    pub fn columns(&self) -> Columns {
        self.columns.clone()
    }

    pub fn values(&self) -> &[Value<'_>] {
        &self.values
    }

    pub fn get<'i>(&self, index: impl IntoValueIndex<'i>) -> &Value<'_> {
        unsafe { self.try_get(index).unwrap_unchecked() }
    }

    pub fn try_get<'i>(&self, index: impl IntoValueIndex<'i>) -> Option<&Value<'_>> {
        match index.into_index() {
            ValueIndex::ColumnName(name) => {
                let i = self.columns.iter().position(|(n, _)| n == name)?;
                self.values.get(i)
            }
            ValueIndex::ColumnIndex(i) => self.values.get(i),
        }
    }

    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, DeError> {
        let deserializer = RowDeserializer { row: self };
        T::deserialize(deserializer)
    }
}

pub enum ValueIndex<'i> {
    ColumnName(&'i str),
    ColumnIndex(usize),
}

pub trait IntoValueIndex<'a> {
    fn into_index(self) -> ValueIndex<'a>;
}

impl<'i> IntoValueIndex<'i> for usize {
    fn into_index(self) -> ValueIndex<'i> {
        ValueIndex::ColumnIndex(self)
    }
}

impl<'i> IntoValueIndex<'i> for &'i str {
    fn into_index(self) -> ValueIndex<'i> {
        ValueIndex::ColumnName(self)
    }
}

impl<'i> IntoValueIndex<'i> for ValueIndex<'i> {
    fn into_index(self) -> ValueIndex<'i> {
        self
    }
}
