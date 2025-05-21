use crate::*;
use std::rc::Rc;
use std::slice::Iter;
use strum::{Display, FromRepr, IntoStaticStr};

// https://github.com/crossdb-org/crossdb/blob/main/include/crossdb.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr, FromRepr)]
#[repr(u32)]
pub enum DataType {
    #[strum(serialize = "NULL")]
    Null = xdb_type_t_XDB_TYPE_NULL,
    #[strum(serialize = "TINYINT")]
    TinyInt = xdb_type_t_XDB_TYPE_TINYINT,
    #[strum(serialize = "SMALLINT")]
    SmallInt = xdb_type_t_XDB_TYPE_SMALLINT,
    #[strum(serialize = "INT")]
    Int = xdb_type_t_XDB_TYPE_INT,
    #[strum(serialize = "BIGINT")]
    BigInt = xdb_type_t_XDB_TYPE_BIGINT,
    #[strum(serialize = "UTINYINT")]
    UTinyInt = xdb_type_t_XDB_TYPE_UTINYINT,
    #[strum(serialize = "USMALLINT")]
    USmallInt = xdb_type_t_XDB_TYPE_USMALLINT,
    #[strum(serialize = "UINT")]
    UInt = xdb_type_t_XDB_TYPE_UINT,
    #[strum(serialize = "UBIGINT")]
    UBigInt = xdb_type_t_XDB_TYPE_UBIGINT,
    #[strum(serialize = "FLOAT")]
    Float = xdb_type_t_XDB_TYPE_FLOAT,
    #[strum(serialize = "DOUBLE")]
    Double = xdb_type_t_XDB_TYPE_DOUBLE,
    #[strum(serialize = "TIMESTAMP")]
    Timestamp = xdb_type_t_XDB_TYPE_TIMESTAMP,
    #[strum(serialize = "CHAR")]
    Char = xdb_type_t_XDB_TYPE_CHAR,
    #[strum(serialize = "BINARY")]
    Binary = xdb_type_t_XDB_TYPE_BINARY,
    #[strum(serialize = "VCHAR")]
    VChar = xdb_type_t_XDB_TYPE_VCHAR,
    #[strum(serialize = "VBINARY")]
    VBinary = xdb_type_t_XDB_TYPE_VBINARY,
    #[strum(serialize = "BOOL")]
    Bool = xdb_type_t_XDB_TYPE_BOOL,
    #[strum(serialize = "INET")]
    Inet = xdb_type_t_XDB_TYPE_INET,
    #[strum(serialize = "MAC")]
    Mac = xdb_type_t_XDB_TYPE_MAC,
    #[strum(serialize = "JSON")]
    Json = xdb_type_t_XDB_TYPE_JSON,
    #[strum(serialize = "ARRAY")]
    Array = xdb_type_t_XDB_TYPE_ARRAY,
    #[strum(serialize = "MAX")]
    Max = xdb_type_t_XDB_TYPE_MAX,
}

impl DataType {
    unsafe fn from_res(ptr: *mut xdb_res_t, col: u16) -> Self {
        let t = xdb_column_type(ptr, col);
        match Self::from_repr(t) {
            Some(t) => t,
            None => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Columns {
    inner: Rc<Vec<Column>>,
}

impl Columns {
    pub(crate) unsafe fn from_res(ptr: *mut xdb_res_t) -> Self {
        let count = xdb_column_count(ptr);
        let mut columns = Vec::with_capacity(count as usize);
        for i in 0..(count as u16) {
            unsafe {
                let name = CStr::from_ptr(xdb_column_name(ptr, i))
                    .to_str()
                    .unwrap()
                    .to_string();
                let datatype = DataType::from_res(ptr, i);
                columns.push(Column::new(name, datatype));
            }
        }
        Self {
            inner: Rc::new(columns),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn name(&self, i: usize) -> &str {
        self.inner[i].name()
    }

    pub fn datatype(&self, i: usize) -> DataType {
        self.inner[i].datatype()
    }

    pub fn iter(&self) -> ColumnsIter {
        ColumnsIter {
            inner: self.inner.iter(),
        }
    }

    pub fn into_inner(self) -> Option<Vec<Column>> {
        Rc::into_inner(self.inner)
    }
}

pub struct ColumnsIter<'a> {
    inner: Iter<'a, Column>,
}

impl<'a> Iterator for ColumnsIter<'a> {
    type Item = &'a Column;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a> IntoIterator for &'a Columns {
    type Item = &'a Column;
    type IntoIter = ColumnsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    datatype: DataType,
}

impl Column {
    pub fn new(name: String, datatype: DataType) -> Self {
        Self { name, datatype }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn datatype(&self) -> DataType {
        self.datatype
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.datatype)
    }
}
