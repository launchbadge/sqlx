use std::ffi::CStr;

use libsqlite3_sys::{
    sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_double, sqlite3_column_int,
    sqlite3_column_int64, sqlite3_column_text, sqlite3_column_type, SQLITE_BLOB, SQLITE_FLOAT,
    SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT,
};

use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::types::SqliteType;
use crate::sqlite::SqliteConnection;
use core::slice;

pub struct SqliteResultValue<'c> {
    pub(super) index: usize,
    pub(super) statement: Option<usize>,
    pub(super) connection: &'c SqliteConnection,
}

impl SqliteResultValue<'_> {
    fn statement(&self) -> &SqliteStatement {
        self.connection.statement(self.statement)
    }
}

// https://www.sqlite.org/c3ref/column_blob.html
// https://www.sqlite.org/capi3ref.html#sqlite3_column_blob

// These routines return information about a single column of the current result row of a query.

impl<'c> SqliteResultValue<'c> {
    pub(crate) fn r#type(&self) -> SqliteType {
        #[allow(unsafe_code)]
        let type_code =
            unsafe { sqlite3_column_type(self.statement().handle(), self.index as i32) };

        match type_code {
            SQLITE_INTEGER => SqliteType::Integer,
            SQLITE_FLOAT => SqliteType::Float,
            SQLITE_BLOB => SqliteType::Blob,
            SQLITE_NULL => SqliteType::Null,
            SQLITE_TEXT => SqliteType::Text,

            _ => unreachable!(),
        }
    }

    pub(crate) fn int(&self) -> i32 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_int(self.statement().handle(), self.index as i32)
        }
    }

    pub(crate) fn int64(&self) -> i64 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_int64(self.statement().handle(), self.index as i32)
        }
    }

    pub(crate) fn double(&self) -> f64 {
        #[allow(unsafe_code)]
        unsafe {
            sqlite3_column_double(self.statement().handle(), self.index as i32)
        }
    }

    pub(crate) fn text(&self) -> crate::Result<&'c str> {
        #[allow(unsafe_code)]
        let raw = unsafe {
            let ptr =
                sqlite3_column_text(self.statement().handle(), self.index as i32) as *const i8;

            debug_assert!(!ptr.is_null());

            CStr::from_ptr(ptr)
        };

        raw.to_str().map_err(crate::Error::decode)
    }

    pub(crate) fn blob(&self) -> crate::Result<&'c [u8]> {
        let index = self.index as i32;

        #[allow(unsafe_code)]
        let ptr = unsafe { sqlite3_column_blob(self.statement().handle(), index) };

        #[allow(unsafe_code)]
        let len = unsafe { sqlite3_column_bytes(self.statement().handle(), index) };

        #[allow(unsafe_code)]
        let raw = unsafe { slice::from_raw_parts(ptr as *const u8, len as usize) };

        Ok(raw)
    }
}
