use std::ffi::NulError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CString error: {0}")]
    CString(#[from] NulError),
    #[error("UTF8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Query error: {0}, {1}")]
    Query(u16, String),
}
