use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Null,
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Char(&'a str),
}

impl<'a> Value<'a> {
    // TODO: If you know the detailed format, you can access the pointer directly
    // https://crossdb.org/client/api-c/#xdb_column_int
    pub(crate) unsafe fn from_result(
        meta: u64,
        row: *mut xdb_row_t,
        col: u16,
        t: DataType,
    ) -> Value<'a> {
        match t {
            DataType::TinyInt => Value::I8(xdb_column_int(meta, row, col) as _),
            DataType::SmallInt => Value::I16(xdb_column_int(meta, row, col) as _),
            DataType::Int => Value::I32(xdb_column_int(meta, row, col) as _),
            DataType::BigInt => Value::I64(xdb_column_int64(meta, row, col)),
            DataType::Float => Value::F32(xdb_column_float(meta, row, col)),
            DataType::Double => Value::F64(xdb_column_double(meta, row, col)),
            DataType::Char => {
                let s = CStr::from_ptr(xdb_column_str(meta, row, col));
                Value::Char(s.to_str().unwrap())
            }
            _ => unimplemented!(),
        }
    }
}
