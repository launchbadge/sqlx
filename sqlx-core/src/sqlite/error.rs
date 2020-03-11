use crate::error::DatabaseError;
use libc::c_int;
use std::ffi::CStr;
use libsqlite3_sys::{sqlite3, sqlite3_errstr};

pub struct SqliteError {
    #[allow(dead_code)]
    code: c_int,
    message: String,
}

impl SqliteError {
    pub(crate) fn new(code: c_int) -> Self {
        #[allow(unsafe_code)]
        let message = unsafe {
            CStr::from_ptr(sqlite3_errstr(code))
        };

        Self { code, message: message.to_string_lossy().into_owned() }
    }
}

impl DatabaseError for SqliteError {
    fn message(&self) -> &str {
        &self.message
    }
}

impl_fmt_error!(SqliteError);
