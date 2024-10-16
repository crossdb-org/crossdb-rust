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
        self.try_get(index).expect("Row index out of bounds")
    }

    pub fn try_get<'i>(&self, index: impl IntoValueIndex<'i>) -> Option<&Value<'_>> {
        match index.into_index() {
            ValueIndex::ColumnName(name) => {
                let i = self.columns.iter().position(|c| c.name() == name)?;
                self.values.get(i)
            }
            ValueIndex::ColumnIndex(i) => self.values.get(i),
        }
    }

    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, DeError> {
        T::deserialize(RowDeserializer::new(self))
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
