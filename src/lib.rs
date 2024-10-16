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
mod params;

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
    res: xdb_res_t,
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
        let res = *ptr;
        if res.errcode != xdb_errno_e_XDB_OK as u16 {
            let msg = CStr::from_ptr(xdb_errmsg(ptr)).to_str()?.to_string();
            return Err(Error::Query(res.errcode, msg));
        }
        Ok(Self {
            res,
            ptr,
            columns: Columns::from_res(ptr),
        })
    }

    pub fn column_count(&self) -> usize {
        self.res.col_count as usize
    }

    pub fn row_count(&self) -> usize {
        self.res.row_count as usize
    }

    pub fn affected_rows(&self) -> u64 {
        self.res.affected_rows
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

    pub fn fetch_rows_as<'a, T: DeserializeOwned>(&mut self) -> Result<Vec<T>, DeError> {
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
            let mut values = Vec::with_capacity(self.column_count());
            for col in 0..self.column_count() {
                let value = Value::from_result(
                    self.res.col_meta,
                    row,
                    col as u16,
                    self.columns.datatype(col),
                );
                values.push(value);
            }
            Some(values)
        }
    }
}
