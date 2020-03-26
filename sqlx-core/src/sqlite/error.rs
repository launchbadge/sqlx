use crate::error::DatabaseError;

use bitflags::_core::str::from_utf8_unchecked;
use libsqlite3_sys::{sqlite3, sqlite3_errmsg, sqlite3_extended_errcode};
use std::error::Error as StdError;
use std::ffi::CStr;
use std::fmt::{self, Display};
use std::os::raw::c_int;

#[derive(Debug)]
pub struct SqliteError {
    code: String,
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
            code: code.to_string(),
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

    fn code(&self) -> Option<&str> {
        Some(&self.code)
    }

    fn as_ref_err(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_mut_err(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_box_err(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }
}

impl StdError for SqliteError {}

impl From<SqliteError> for crate::Error {
    fn from(err: SqliteError) -> Self {
        crate::Error::Database(Box::new(err))
    }
}

#[test]
fn test_error_downcasting() {
    let error = SqliteError {
        code: "SQLITE_ERR_SOMETHING".into(),
        message: "Some hypothetical error message".into(),
    };

    let error = crate::Error::from(error);

    let db_err = match error {
        crate::Error::Database(db_err) => db_err,
        e => panic!("expected Error::Database, got {:?}", e),
    };

    assert_eq!(
        &db_err.downcast_ref::<SqliteError>().code,
        "SQLITE_ERR_SOMETHING"
    );
    assert_eq!(
        db_err.downcast::<SqliteError>().code,
        "SQLITE_ERR_SOMETHING"
    );
}
