use params::Params;

use crate::*;

pub struct Statement {
    pub(crate) ptr: *mut xdb_stmt_t,
}

impl Drop for Statement {
    fn drop(&mut self) {
        unsafe {
            xdb_stmt_close(self.ptr);
        }
    }
}

impl Statement {
    pub fn query(&self, params: impl IntoParams) -> Result<Query> {
        unsafe {
            let params = params.into_params()?;
            if let Params::Positional(params) = params {
                if !params.is_empty() {
                    self.clear_bindings()?;
                    Params::bind(self.ptr, params)?;
                }
            }
            let ptr = xdb_stmt_exec(self.ptr);
            Query::from_res(ptr)
        }
    }

    pub fn execute(&self, params: impl IntoParams) -> Result<u64> {
        self.query(params).map(|q| q.affected_rows())
    }

    pub fn clear_bindings(&self) -> Result<()> {
        let ret = unsafe { xdb_clear_bindings(self.ptr) };
        match ret {
            0 => Ok(()),
            _ => Err(Error::ClearBindings),
        }
    }
}
