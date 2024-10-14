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
mod de;
mod error;
mod row;
mod value;

pub use column::{Columns, DataType};
pub use error::{Error, Result};
pub use row::Row;
pub use value::Value;

use crossdb_sys::*;
use params::{IntoParams, Value as ParamValue};
use std::ffi::{CStr, CString};
use std::fmt::Display;
mod params;

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
                res,
                ptr,
                columns: Columns::from_res(ptr),
            })
        }
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

    // TODO: LRU cache
    pub fn prepare<S: AsRef<str>>(&mut self, sql: S) -> Result<Stmt> {
        unsafe {
            let sql = CString::new(sql.as_ref())?;
            let ptr = xdb_stmt_prepare(self.ptr, sql.as_ptr());
            Ok(Stmt { ptr })
        }
    }
}

pub struct Stmt {
    ptr: *mut xdb_stmt_t,
}

impl Drop for Stmt {
    fn drop(&mut self) {
        unsafe {
            xdb_stmt_close(self.ptr);
        }
    }
}

impl Stmt {
    pub fn exec(&self, params: impl IntoParams) -> Result<ExecResult> {
        unsafe {
            let ret = xdb_clear_bindings(self.ptr);
            if ret != 0 {
                return Err(Error::ClearBindings);
            }
            params.into_params()?.bind(self.ptr)?;
            let ptr = xdb_stmt_exec(self.ptr);
            let res = *ptr;
            if res.errcode as u32 != xdb_errno_e_XDB_OK {
                let msg = CStr::from_ptr(xdb_errmsg(ptr)).to_str()?.to_string();
                return Err(Error::Query(res.errcode, msg));
            }
            Ok(ExecResult {
                res,
                ptr,
                columns: Columns::from_res(ptr),
            })
        }
    }
}

#[derive(Debug)]
pub struct ExecResult {
    res: xdb_res_t,
    ptr: *mut xdb_res_t,
    columns: Columns,
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

    pub fn columns(&self) -> &Columns {
        &self.columns
    }

    pub fn fetch_row(&mut self) -> Option<Row<'_>> {
        let columns = self.columns.clone();
        let values = self.inner_fetch_row_values()?;
        Some(Row { columns, values })
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
