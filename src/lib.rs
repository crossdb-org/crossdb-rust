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
            let types = ColumnType::all(&res);
            Ok(ExecResult { res, ptr, types })
        }
    }

    pub fn begin(&self) -> bool {
        unsafe {
            let status = xdb_begin(self.ptr);
            status as u32 == xdb_errno_e_XDB_OK
        }
    }

    pub fn commit(&self) -> bool {
        unsafe {
            let status = xdb_commit(self.ptr);
            status as u32 == xdb_errno_e_XDB_OK
        }
    }

    pub fn rollback(&self) -> bool {
        unsafe {
            let status = xdb_rollback(self.ptr);
            status as u32 == xdb_errno_e_XDB_OK
        }
    }
}

#[derive(Debug)]
pub struct ExecResult {
    res: xdb_res_t,
    ptr: *mut xdb_res_t,
    types: Vec<ColumnType>,
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
        self.res.col_count as usize
    }

    pub fn row_count(&self) -> usize {
        self.res.row_count as usize
    }

    pub fn affected_rows(&self) -> u64 {
        self.res.affected_rows
    }

    pub fn column_name<'a>(&'a self, i: usize) -> &'a str {
        unsafe {
            let name = xdb_column_name(self.res.col_meta, i as u16);
            CStr::from_ptr(name).to_str().unwrap()
        }
    }

    pub fn column_type(&self, i: usize) -> ColumnType {
        self.types[i]
    }

    pub fn fetch_row(&mut self) -> Option<Vec<Value<'_>>> {
        unsafe {
            let row = xdb_fetch_row(self.ptr);
            if row.is_null() {
                return None;
            }
            let mut values = Vec::with_capacity(self.column_count());
            for col in 0..self.column_count() {
                let value =
                    Value::from_result(self.res.col_meta, row, col as u16, self.column_type(col));
                values.push(value);
            }
            Some(values)
        }
    }
}
