use core::slice;

use std::ffi::CStr;
use std::str::from_utf8_unchecked;

use libsqlite3_sys::{
    sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_double, sqlite3_column_int,
    sqlite3_column_int64, sqlite3_column_text, sqlite3_column_type, SQLITE_BLOB, SQLITE_FLOAT,
    SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT,
};

use crate::sqlite::statement::Statement;
use crate::sqlite::type_info::SqliteType;
use crate::sqlite::{Sqlite, SqliteTypeInfo};
use crate::value::RawValue;

pub struct SqliteValue<'c> {
    pub(super) index: i32,
    pub(super) statement: &'c Statement,
}

// https://www.sqlite.org/c3ref/column_blob.html
// https://www.sqlite.org/capi3ref.html#sqlite3_column_blob

// These routines return information about a single column of the current result row of a query.

impl<'c> SqliteValue<'c> {
    /// Returns true if the value should be intrepreted as NULL.
    pub(super) fn is_null(&self) -> bool {
        self.r#type().is_none()
    }

    fn r#type(&self) -> Option<SqliteType> {
        let type_code = unsafe {
            if let Some(handle) = self.statement.handle() {
                sqlite3_column_type(handle, self.index)
            } else {
                // unreachable: null statements do not have any values to type
                return None;
            }
        };

        // SQLITE_INTEGER, SQLITE_FLOAT, SQLITE_TEXT, SQLITE_BLOB, or SQLITE_NULL
        match type_code {
            SQLITE_INTEGER => Some(SqliteType::Integer),
            SQLITE_FLOAT => Some(SqliteType::Float),
            SQLITE_TEXT => Some(SqliteType::Text),
            SQLITE_BLOB => Some(SqliteType::Blob),
            SQLITE_NULL => None,

            _ => unreachable!("received unexpected column type: {}", type_code),
        }
    }

    /// Returns the 32-bit INTEGER result.
    pub(super) fn int(&self) -> i32 {
        unsafe {
            self.statement
                .handle()
                .map_or(0, |handle| sqlite3_column_int(handle, self.index))
        }
    }

    /// Returns the 64-bit INTEGER result.
    pub(super) fn int64(&self) -> i64 {
        unsafe {
            self.statement
                .handle()
                .map_or(0, |handle| sqlite3_column_int64(handle, self.index))
        }
    }

    /// Returns the 64-bit, REAL result.
    pub(super) fn double(&self) -> f64 {
        unsafe {
            self.statement
                .handle()
                .map_or(0.0, |handle| sqlite3_column_double(handle, self.index))
        }
    }

    /// Returns the UTF-8 TEXT result.
    pub(super) fn text(&self) -> Option<&'c str> {
        unsafe {
            self.statement.handle().and_then(|handle| {
                let ptr = sqlite3_column_text(handle, self.index);

                if ptr.is_null() {
                    None
                } else {
                    Some(from_utf8_unchecked(CStr::from_ptr(ptr as _).to_bytes()))
                }
            })
        }
    }

    fn bytes(&self) -> usize {
        // Returns the size of the result in bytes.
        unsafe {
            self.statement
                .handle()
                .map_or(0, |handle| sqlite3_column_bytes(handle, self.index)) as usize
        }
    }

    /// Returns the BLOB result.
    pub(super) fn blob(&self) -> &'c [u8] {
        let ptr = unsafe {
            if let Some(handle) = self.statement.handle() {
                sqlite3_column_blob(handle, self.index)
            } else {
                // Null statements do not exist
                return &[];
            }
        };

        if ptr.is_null() {
            // Empty BLOBs are received as null pointers
            return &[];
        }

        unsafe { slice::from_raw_parts(ptr as *const u8, self.bytes()) }
    }
}

impl<'c> RawValue<'c> for SqliteValue<'c> {
    type Database = Sqlite;

    fn type_info(&self) -> Option<SqliteTypeInfo> {
        Some(SqliteTypeInfo {
            r#type: self.r#type()?,
            affinity: None,
        })
    }
}
