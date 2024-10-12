#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

mod crossdb_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod column;
mod error;
mod value;

pub use column::ColumnType;
pub use error::{Error, Result};
pub use value::Value;

use crossdb_sys::*;
use std::ffi::{CStr, CString};
use std::fmt::Display;

#[derive(Debug)]
pub struct Connection {
    ptr: *mut xdb_conn_t,
}

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
        Ok(Self { ptr })
    }

    pub fn open_with_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    pub fn exec<S: AsRef<str>>(&self, sql: S) -> Result<ExecResult> {
        let sql = CString::new(sql.as_ref())?;
        unsafe {
            let ptr = xdb_exec(self.ptr, sql.as_ptr());
            let res = *ptr;
            if res.errcode as u32 != xdb_errno_e_XDB_OK {
                let msg = CStr::from_ptr(xdb_errmsg(ptr)).to_str()?.to_string();
                return Err(Error::Query(res.errcode, msg));
            }
            Ok(ExecResult {
                ptr,
                col_meta: res.col_meta,
                column_count: res.col_count as usize,
                row_count: res.row_count as usize,
                column_types: ColumnType::all(&res),
                row_index: 0,
            })
        }
    }
}

#[derive(Debug)]
pub struct ExecResult {
    ptr: *mut xdb_res_t,
    col_meta: u64,
    column_count: usize,
    row_count: usize,
    column_types: Vec<ColumnType>,
    row_index: usize,
}

impl Drop for ExecResult {
    fn drop(&mut self) {
        unsafe {
            xdb_free_result(self.ptr);
        }
    }
}

impl ExecResult {
    pub fn column_count(&self) -> usize {
        self.column_count
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn column_name<'a>(&'a self, i: usize) -> &'a str {
        unsafe {
            let name = xdb_column_name(self.col_meta, i as u16);
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    pub fn column_type(&self, i: usize) -> ColumnType {
        self.column_types[i]
    }
}

impl<'a> Iterator for &'a mut ExecResult {
    type Item = Vec<Value<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_count <= self.row_index {
            return None;
        }
        let mut values = Vec::with_capacity(self.column_count);
        unsafe {
            let y = xdb_fetch_row(self.ptr);
            for x in 0..self.column_count {
                let value = Value::from_result(self.col_meta, y, x as u16, self.column_type(x));
                values.push(value);
            }
        }
        self.row_index += 1;
        Some(values)
    }
}
