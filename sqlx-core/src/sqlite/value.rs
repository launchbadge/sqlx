use core::slice;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use libsqlite3_sys::{
    sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_double, sqlite3_column_int,
    sqlite3_column_int64, sqlite3_column_text, sqlite3_column_type, SQLITE_BLOB, SQLITE_FLOAT,
    SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT,
};

use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::types::SqliteType;

pub struct SqliteResultValue<'c> {
    index: usize,
    statement: &'c SqliteStatement,
}

impl<'c> SqliteResultValue<'c> {
    #[inline]
    pub(super) fn new(statement: &'c SqliteStatement, index: usize) -> Self {
        Self { statement, index }
    }
}

// https://www.sqlite.org/c3ref/column_blob.html
// https://www.sqlite.org/capi3ref.html#sqlite3_column_blob

// These routines return information about a single column of the current result row of a query.

impl<'c> SqliteResultValue<'c> {
    /// Returns the initial data type of the result column.
    pub(super) fn r#type(&self) -> SqliteType {
        #[allow(unsafe_code)]
        let type_code = unsafe { sqlite3_column_type(self.statement.handle(), self.index as i32) };

        match type_code {
            SQLITE_INTEGER => SqliteType::Integer,
            SQLITE_FLOAT => SqliteType::Float,
            SQLITE_BLOB => SqliteType::Blob,
            SQLITE_NULL => SqliteType::Null,
            SQLITE_TEXT => SqliteType::Text,

            _ => unreachable!(),
        }
    }

    /// Returns the 32-bit INTEGER result.
    pub(super) fn int(&self) -> i32 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_int(self.statement.handle(), self.index as i32)
        }
    }

    /// Returns the 64-bit INTEGER result.
    pub(super) fn int64(&self) -> i64 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_int64(self.statement.handle(), self.index as i32)
        }
    }

    /// Returns the 64-bit, REAL result.
    pub(super) fn double(&self) -> f64 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_double(self.statement.handle(), self.index as i32)
        }
    }

    /// Returns the UTF-8 TEXT result.
    pub(super) fn text(&self) -> &'c str {
        #[allow(unsafe_code)]
        unsafe {
            let ptr = sqlite3_column_text(self.statement.handle(), self.index as i32) as *const i8;

            debug_assert!(!ptr.is_null());

            from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes())
        }
    }

    /// Returns the BLOB result.
    pub(super) fn blob(&self) -> &'c [u8] {
        let index = self.index as i32;

        #[allow(unsafe_code)]
        let ptr = unsafe { sqlite3_column_blob(self.statement.handle(), index) };

        // Returns the size of the BLOB result in bytes.
        #[allow(unsafe_code)]
        let len = unsafe { sqlite3_column_bytes(self.statement.handle(), index) };

        #[allow(unsafe_code)]
        unsafe {
            slice::from_raw_parts(ptr as *const u8, len as usize)
        }
    }
}
