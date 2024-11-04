use crate::*;
use std::rc::Rc;
use std::slice::Iter;

// https://github.com/crossdb-org/crossdb/blob/main/include/crossdb.h
#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Null,
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    UTinyInt,
    USmallInt,
    UInt,
    UBigInt,
    Float,
    Double,
    Timestamp,
    Char,
    Binary,
    VChar,
    VBinary,
    Bool,
    Max,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Null => write!(f, "NULL"),
            DataType::TinyInt => write!(f, "TINYINT"),
            DataType::SmallInt => write!(f, "SMALLINT"),
            DataType::Int => write!(f, "INT"),
            DataType::BigInt => write!(f, "BIGINT"),
            DataType::UTinyInt => write!(f, "UTINYINT"),
            DataType::USmallInt => write!(f, "USMALLINT"),
            DataType::UInt => write!(f, "UINT"),
            DataType::UBigInt => write!(f, "UBIGINT"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Char => write!(f, "CHAR"),
            DataType::Binary => write!(f, "BINARY"),
            DataType::VChar => write!(f, "VCHAR"),
            DataType::VBinary => write!(f, "VBINARY"),
            DataType::Bool => write!(f, "BOOL"),
            DataType::Max => write!(f, "MAX"),
        }
    }
}

impl DataType {
    #[allow(non_upper_case_globals)]
    unsafe fn from_meta(meta: u64, col: u16) -> Self {
        let t = xdb_column_type(meta, col);
        match t {
            xdb_type_t_XDB_TYPE_NULL => Self::Null,
            xdb_type_t_XDB_TYPE_TINYINT => Self::TinyInt,
            xdb_type_t_XDB_TYPE_SMALLINT => Self::SmallInt,
            xdb_type_t_XDB_TYPE_INT => Self::Int,
            xdb_type_t_XDB_TYPE_BIGINT => Self::BigInt,
            xdb_type_t_XDB_TYPE_UTINYINT => Self::UTinyInt,
            xdb_type_t_XDB_TYPE_USMALLINT => Self::USmallInt,
            xdb_type_t_XDB_TYPE_UINT => Self::UInt,
            xdb_type_t_XDB_TYPE_UBIGINT => Self::UBigInt,
            xdb_type_t_XDB_TYPE_FLOAT => Self::Float,
            xdb_type_t_XDB_TYPE_DOUBLE => Self::Double,
            xdb_type_t_XDB_TYPE_TIMESTAMP => Self::Timestamp,
            xdb_type_t_XDB_TYPE_CHAR => Self::Char,
            xdb_type_t_XDB_TYPE_BINARY => Self::Binary,
            xdb_type_t_XDB_TYPE_VCHAR => Self::VChar,
            xdb_type_t_XDB_TYPE_VBINARY => Self::VBinary,
            xdb_type_t_XDB_TYPE_BOOL => Self::Bool,
            xdb_type_t_XDB_TYPE_MAX => Self::Max,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Columns {
    inner: Rc<Vec<Column>>,
}

impl Columns {
    pub(crate) unsafe fn from_res(ptr: *mut xdb_res_t) -> Self {
        let res = *ptr;
        let mut columns = Vec::with_capacity(res.col_count as usize);
        for i in 0..res.col_count {
            unsafe {
                let name = CStr::from_ptr(xdb_column_name(res.col_meta, i))
                    .to_str()
                    .unwrap()
                    .to_string();
                let datatype = DataType::from_meta(res.col_meta, i);
                columns.push(Column::new(name, datatype));
            }
        }
        Self {
            inner: Rc::new(columns),
        }
    }

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
