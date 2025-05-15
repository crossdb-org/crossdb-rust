#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    improper_ctypes
)]

mod crossdb_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod column;
mod de;
mod error;
mod params;
mod row;
mod statement;
mod value;

pub use column::{Column, Columns, ColumnsIter, DataType};
pub use error::{Error, Result};
pub use params::{IntoParams, Params, Value as ParamValue};
pub use row::{IntoValueIndex, Row, ValueIndex};
pub use statement::Statement;
pub use value::Value;

use crossdb_sys::*;
use lru::LruCache;
use serde::de::{value::Error as DeError, DeserializeOwned};
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::num::NonZeroUsize;
use std::slice::from_raw_parts;

pub fn version() -> &'static str {
    unsafe { CStr::from_ptr(xdb_version()).to_str().unwrap() }
}

#[derive(Debug)]
pub struct Connection {
    ptr: *mut xdb_conn_t,
    cache: LruCache<CString, Statement>,
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            xdb_close(self.ptr);
        }
    }
}

impl Connection {
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let path = CString::new(path.as_ref())?;
        let ptr = unsafe { xdb_open(path.as_ptr()) };
        let cap = NonZeroUsize::new(256).unwrap();
        Ok(Self {
            ptr,
            cache: LruCache::new(cap),
        })
    }

    pub fn open_with_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    pub fn current_database(&self) -> Result<&str> {
        unsafe {
            let ptr = xdb_curdb(self.ptr);
            let db = CStr::from_ptr(ptr).to_str()?;
            Ok(db)
        }
    }

    pub fn query<S: AsRef<str>>(&self, sql: S) -> Result<Query> {
        let sql = CString::new(sql.as_ref())?;
        unsafe {
            let ptr = xdb_exec(self.ptr, sql.as_ptr());
            Query::from_res(ptr)
        }
    }

    pub fn execute<S: AsRef<str>>(&self, sql: S) -> Result<u64> {
        self.query(sql).map(|q| q.affected_rows())
    }

    pub fn begin(&self) -> bool {
        unsafe { xdb_begin(self.ptr) == 0 }
    }

    pub fn commit(&self) -> bool {
        unsafe { xdb_commit(self.ptr) == 0 }
    }

    pub fn rollback(&self) -> bool {
        unsafe { xdb_rollback(self.ptr) == 0 }
    }

    pub fn prepare<S: AsRef<str>>(&mut self, sql: S) -> Result<&Statement> {
        let sql = CString::new(sql.as_ref())?;
        let sql_ptr = sql.as_ptr();
        let stmt = self.cache.get_or_insert(sql, || {
            let ptr = unsafe { xdb_stmt_prepare(self.ptr, sql_ptr) };
            Statement { ptr }
        });
        Ok(stmt)
    }

    pub fn resize_statement_cache(&mut self, capacity: usize) {
        assert_ne!(capacity, 0);
        self.cache.resize(NonZeroUsize::new(capacity).unwrap());
    }

    pub fn clear_statement_cache(&mut self) {
        self.cache.clear();
    }
}

#[derive(Debug)]
pub struct Query {
    ptr: *mut xdb_res_t,
    columns: Columns,
}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe {
            xdb_free_result(self.ptr);
        }
    }
}

unsafe impl Send for Query {}
unsafe impl Sync for Query {}

impl Query {
    pub(crate) unsafe fn from_res(ptr: *mut xdb_res_t) -> Result<Self> {
        let code = xdb_errcode(ptr);
        if code != xdb_errno_e_XDB_OK {
            let msg = CStr::from_ptr(xdb_errmsg(ptr)).to_str()?.to_string();
            return Err(Error::Query(code, msg));
        }
        Ok(Self {
            ptr,
            columns: Columns::from_res(ptr),
        })
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        unsafe { xdb_row_count(self.ptr) as usize }
    }

    pub fn affected_rows(&self) -> u64 {
        unsafe { xdb_affected_rows(self.ptr) as u64 }
    }

    pub fn columns(&self) -> &Columns {
        &self.columns
    }

    pub fn fetch_row(&mut self) -> Option<Row<'_>> {
        let columns = self.columns.clone();
        let values = self.inner_fetch_row_values()?;
        Some(Row { columns, values })
    }

    pub fn fetch_row_as<T: DeserializeOwned>(&mut self) -> Option<Result<T, DeError>> {
        self.fetch_row().map(|row| row.deserialize())
    }

    pub fn fetch_rows_as<T: DeserializeOwned>(&mut self) -> Result<Vec<T>, DeError> {
        let mut rows = Vec::with_capacity(self.row_count());
        while let Some(row) = self.fetch_row() {
            rows.push(row.deserialize()?);
        }
        Ok(rows)
    }

    fn inner_fetch_row_values(&mut self) -> Option<Vec<Value<'_>>> {
        unsafe {
            let row = xdb_fetch_row(self.ptr);
            if row.is_null() {
                return None;
            }
            let count = self.columns.len();
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                let v = Value::from_ptr(self.ptr, row, i as u16, self.columns.datatype(i));
                values.push(v);
            }
            Some(values)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let mut conn = Connection::open_with_memory().unwrap();

        conn.execute("CREATE TABLE IF NOT EXISTS users(id INT, name VARCHAR(255), age TINYINT);")
            .unwrap();
        let stmt = conn
            .prepare("INSERT INTO users (id, name, age) values (?, ?, ?);")
            .unwrap();

        stmt.execute((1, "Alex", 18)).unwrap();
        stmt.execute((2, "Thorne", 22)).unwrap();
        stmt.execute((3, "Ryder", 36)).unwrap();

        let mut query = conn.query("SELECT * FROM users;").unwrap();

        assert_eq!(query.row_count(), 3);
        assert_eq!(query.column_count(), 3);

        assert_eq!(query.columns.name(0), "id");
        assert_eq!(query.columns.datatype(0), DataType::Int);
        assert_eq!(query.columns.name(1), "name");
        assert_eq!(query.columns.datatype(1), DataType::VChar);
        assert_eq!(query.columns.name(2), "age");
        assert_eq!(query.columns.datatype(2), DataType::TinyInt);

        let row1 = query.fetch_row().unwrap();
        assert_eq!(row1.get(0), &Value::I32(1));
        // assert_eq!(row1.get(1), &Value::String("Alex"));
        assert_eq!(row1.get(2), &Value::I32(18));

        let row2 = query.fetch_row().unwrap();
        assert_eq!(row2.get(0), &Value::I32(2));
        // assert_eq!(row2.get(1), &Value::String("Thorne"));
        assert_eq!(row2.get(2), &Value::I32(22));

        let row3 = query.fetch_row().unwrap();
        assert_eq!(row3.get(0), &Value::I32(3));
        // assert_eq!(row3.get(1), &Value::String("Ryder"));
        assert_eq!(row3.get(2), &Value::I32(36));

        assert!(query.fetch_row().is_none());

        let affected_rows = conn.execute("DELETE FROM users;").unwrap();
        assert_eq!(affected_rows, 3);
    }
}
