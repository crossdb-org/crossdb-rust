use crate::*;

// https://crossdb.org/client/api-c/#xdb_type_t
#[derive(Debug, Clone, Copy)]
pub enum ColumnType {
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
    Max,
}

impl From<u32> for ColumnType {
    #[allow(non_upper_case_globals)]
    fn from(value: u32) -> Self {
        match value {
            xdb_type_t_XDB_TYPE_NULL => ColumnType::Null,
            xdb_type_t_XDB_TYPE_TINYINT => ColumnType::TinyInt,
            xdb_type_t_XDB_TYPE_SMALLINT => ColumnType::SmallInt,
            xdb_type_t_XDB_TYPE_INT => ColumnType::Int,
            xdb_type_t_XDB_TYPE_BIGINT => ColumnType::BigInt,
            xdb_type_t_XDB_TYPE_UTINYINT => ColumnType::UTinyInt,
            xdb_type_t_XDB_TYPE_USMALLINT => ColumnType::USmallInt,
            xdb_type_t_XDB_TYPE_UINT => ColumnType::UInt,
            xdb_type_t_XDB_TYPE_UBIGINT => ColumnType::UBigInt,
            xdb_type_t_XDB_TYPE_FLOAT => ColumnType::Float,
            xdb_type_t_XDB_TYPE_DOUBLE => ColumnType::Double,
            xdb_type_t_XDB_TYPE_TIMESTAMP => ColumnType::Timestamp,
            xdb_type_t_XDB_TYPE_CHAR => ColumnType::Char,
            xdb_type_t_XDB_TYPE_BINARY => ColumnType::Binary,
            xdb_type_t_XDB_TYPE_VCHAR => ColumnType::VChar,
            xdb_type_t_XDB_TYPE_VBINARY => ColumnType::VBinary,
            xdb_type_t_XDB_TYPE_MAX => ColumnType::Max,
            _ => unreachable!(),
        }
    }
}

impl Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Null => write!(f, "NULL"),
            ColumnType::TinyInt => write!(f, "TINYINT"),
            ColumnType::SmallInt => write!(f, "SMALLINT"),
            ColumnType::Int => write!(f, "INT"),
            ColumnType::BigInt => write!(f, "BIGINT"),
            ColumnType::UTinyInt => write!(f, "UTINYINT"),
            ColumnType::USmallInt => write!(f, "USMALLINT"),
            ColumnType::UInt => write!(f, "UINT"),
            ColumnType::UBigInt => write!(f, "UBIGINT"),
            ColumnType::Float => write!(f, "FLOAT"),
            ColumnType::Double => write!(f, "DOUBLE"),
            ColumnType::Timestamp => write!(f, "TIMESTAMP"),
            ColumnType::Char => write!(f, "CHAR"),
            ColumnType::Binary => write!(f, "BINARY"),
            ColumnType::VChar => write!(f, "VCHAR"),
            ColumnType::VBinary => write!(f, "VBINARY"),
            ColumnType::Max => write!(f, "MAX"),
        }
    }
}

impl ColumnType {
    pub(crate) fn all(res: &xdb_res_t) -> Vec<Self> {
        let mut vec = Vec::with_capacity(res.col_count as usize);
        for i in 0..vec.capacity() {
            unsafe {
                let t = xdb_column_type(res.col_meta, i as u16);
                vec.push(Self::from(t));
            }
        }
        vec
    }
}
