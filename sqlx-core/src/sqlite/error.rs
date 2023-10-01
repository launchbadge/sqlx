use std::error::Error as StdError;
use std::ffi::CStr;
use std::fmt::{self, Display, Formatter};
use std::os::raw::c_int;
use std::{borrow::Cow, str::from_utf8_unchecked};

use libsqlite3_sys::{sqlite3, sqlite3_errmsg, sqlite3_error_offset, sqlite3_extended_errcode};

use crate::error::DatabaseError;

// Error Codes And Messages
// https://www.sqlite.org/c3ref/errcode.html

#[derive(Debug)]
pub struct SqliteError {
    code: c_int,
    offset: Option<usize>,
    message: String,
}

impl SqliteError {
    pub(crate) fn new(handle: *mut sqlite3) -> Self {
        // returns the extended result code even when extended result codes are disabled
        let code: c_int = unsafe { sqlite3_extended_errcode(handle) };

        // sqlite3_error_offset: byte offset of the start of the token that caused the error, or -1 if unknown
        let offset = usize::try_from(unsafe { sqlite3_error_offset(handle) }).ok();

        // return English-language text that describes the error
        let message = unsafe {
            let msg = sqlite3_errmsg(handle);
            debug_assert!(!msg.is_null());

            from_utf8_unchecked(CStr::from_ptr(msg).to_bytes())
        };

        Self {
            code,
            offset,
            message: message.to_owned(),
        }
    }

    /// For errors during extension load, the error message is supplied via a separate pointer
    pub(crate) fn extension(handle: *mut sqlite3, error_msg: &CStr) -> Self {
        let mut err = Self::new(handle);
        err.message = unsafe { from_utf8_unchecked(error_msg.to_bytes()).to_owned() };
        err
    }
}

impl Display for SqliteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // We include the code as some produce ambiguous messages:
        // SQLITE_BUSY: "database is locked"
        // SQLITE_LOCKED: "database table is locked"
        // Sadly there's no function to get the string label back from an error code.
        write!(f, "(code: {}) {}", self.code, self.message)?;
        if let Some(offset) = self.offset {
            write!(f, " (at statement byte offset {})", offset)?;
        };
        Ok(())
    }
}

impl StdError for SqliteError {}

impl DatabaseError for SqliteError {
    /// The extended result code.
    #[inline]
    fn code(&self) -> Option<Cow<'_, str>> {
        Some(format!("{}", self.code).into())
    }

    #[inline]
    fn message(&self) -> &str {
        &self.message
    }

    #[inline]
    fn offset(&self) -> Option<usize> {
        self.offset
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }
}
