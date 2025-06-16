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
            // Here we use the &params
            // Ensure that 'ParamValue::String' is not released before 'xdb_stmt_exec'.
            if let Params::Positional(params) = &params {
                self.clear_bindings()?;
                for (i, p) in params.iter().enumerate() {
                    let i = i as u16 + 1;
                    let ret = match p {
                        ParamValue::Int(v) => xdb_bind_int(self.ptr, i, *v),
                        ParamValue::Int64(v) => xdb_bind_int64(self.ptr, i, *v),
                        ParamValue::Float(v) => xdb_bind_float(self.ptr, i, *v),
                        ParamValue::Double(v) => xdb_bind_double(self.ptr, i, *v),
                        ParamValue::String(v) => {
                            xdb_bind_str2(self.ptr, i, v.as_ptr(), v.as_bytes().len() as i32)
                        }
                    };
                    if ret != 0 {
                        return Err(Error::BindParams);
                    }
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
