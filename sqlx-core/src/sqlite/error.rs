use crate::error::DatabaseError;
use bitflags::_core::str::from_utf8_unchecked;
use libsqlite3_sys::{sqlite3, sqlite3_errmsg, sqlite3_extended_errcode};
use std::ffi::CStr;
use std::fmt::{self, Display};
use std::os::raw::c_int;

#[derive(Debug)]
pub struct SqliteError {
    #[allow(dead_code)]
    code: c_int,
    message: String,
}

// Error Codes And Messages
// https://www.sqlite.org/c3ref/errcode.html

impl SqliteError {
    pub(super) fn from_connection(conn: *mut sqlite3) -> Self {
        #[allow(unsafe_code)]
        let code: c_int = unsafe { sqlite3_extended_errcode(conn) };

        #[allow(unsafe_code)]
        let message = unsafe {
            let err = sqlite3_errmsg(conn);
            debug_assert!(!err.is_null());

            from_utf8_unchecked(CStr::from_ptr(err).to_bytes())
        };

        Self {
            code,
            message: message.to_owned(),
        }
    }
}

impl Display for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}

impl DatabaseError for SqliteError {
    fn message(&self) -> &str {
        &self.message
    }
}
