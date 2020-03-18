use core::slice;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use libsqlite3_sys::{
    sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_double, sqlite3_column_int,
    sqlite3_column_int64, sqlite3_column_text, sqlite3_column_type, SQLITE_NULL,
};

use crate::sqlite::statement::Statement;

pub struct SqliteValue<'c> {
    index: i32,
    statement: &'c Statement,
}

impl<'c> SqliteValue<'c> {
    #[inline]
    pub(super) fn new(statement: &'c Statement, index: usize) -> Self {
        Self {
            statement,
            index: index as i32,
        }
    }
}

// https://www.sqlite.org/c3ref/column_blob.html
// https://www.sqlite.org/capi3ref.html#sqlite3_column_blob

// These routines return information about a single column of the current result row of a query.

impl<'c> SqliteValue<'c> {
    /// Returns true if the value should be intrepreted as NULL.
    pub(super) fn is_null(&self) -> bool {
        #[allow(unsafe_code)]
        let type_code = unsafe { sqlite3_column_type(self.statement.handle(), self.index) };
        type_code == SQLITE_NULL
    }

    /// Returns the 32-bit INTEGER result.
    pub(super) fn int(&self) -> i32 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_int(self.statement.handle(), self.index)
        }
    }

    /// Returns the 64-bit INTEGER result.
    pub(super) fn int64(&self) -> i64 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_int64(self.statement.handle(), self.index)
        }
    }

    /// Returns the 64-bit, REAL result.
    pub(super) fn double(&self) -> f64 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_double(self.statement.handle(), self.index)
        }
    }

    /// Returns the UTF-8 TEXT result.
    pub(super) fn text(&self) -> Option<&'c str> {
        #[allow(unsafe_code)]
        unsafe {
            let ptr = sqlite3_column_text(self.statement.handle(), self.index) as *const i8;

            if ptr.is_null() {
                None
            } else {
                Some(from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes()))
            }
        }
    }

    fn bytes(&self) -> usize {
        // Returns the size of the result in bytes.
        #[allow(unsafe_code)]
        let len = unsafe { sqlite3_column_bytes(self.statement.handle(), self.index) };
        len as usize
    }

    /// Returns the BLOB result.
    pub(super) fn blob(&self) -> &'c [u8] {
        #[allow(unsafe_code)]
        let ptr = unsafe { sqlite3_column_blob(self.statement.handle(), self.index) };

        if ptr.is_null() {
            // Empty BLOBs are received as null pointers
            return &[];
        }

        #[allow(unsafe_code)]
        unsafe {
            slice::from_raw_parts(ptr as *const u8, self.bytes())
        }
    }
}
