use core::ffi::c_void;
use core::mem;

use std::os::raw::c_int;

use libsqlite3_sys::{
    sqlite3_bind_blob, sqlite3_bind_double, sqlite3_bind_int, sqlite3_bind_int64,
    sqlite3_bind_null, sqlite3_bind_text, SQLITE_OK, SQLITE_TRANSIENT,
};

use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::Sqlite;
use crate::sqlite::SqliteError;
use crate::types::Type;

#[derive(Debug, Clone)]
pub enum SqliteArgumentValue {
    Null,

    // TODO: Take by reference to remove the allocation
    Text(String),

    // TODO: Take by reference to remove the allocation
    Blob(Vec<u8>),

    Double(f64),

    Int(i32),

    Int64(i64),
}

#[derive(Default)]
pub struct SqliteArguments {
    index: usize,
    values: Vec<SqliteArgumentValue>,
}

impl SqliteArguments {
    pub(crate) fn next(&mut self) -> Option<SqliteArgumentValue> {
        if self.index >= self.values.len() {
            return None;
        }

        let mut value = SqliteArgumentValue::Null;
        mem::swap(&mut value, &mut self.values[self.index]);

        self.index += 1;
        Some(value)
    }
}

impl Arguments for SqliteArguments {
    type Database = Sqlite;

    fn reserve(&mut self, len: usize, _size_hint: usize) {
        self.values.reserve(len);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<Self::Database> + Type<Self::Database>,
    {
        value.encode(&mut self.values);
    }
}

impl SqliteArgumentValue {
    pub(super) fn bind(&self, statement: &mut SqliteStatement, index: usize) -> crate::Result<()> {
        // TODO: Handle error of trying to bind too many parameters here
        let index = index as c_int;

        // https://sqlite.org/c3ref/bind_blob.html
        #[allow(unsafe_code)]
        let status: c_int = match self {
            SqliteArgumentValue::Blob(value) => {
                // TODO: Handle bytes that are too large
                let bytes = value.as_slice();
                let bytes_ptr = bytes.as_ptr() as *const c_void;
                let bytes_len = bytes.len() as i32;

                unsafe {
                    sqlite3_bind_blob(
                        statement.handle(),
                        index,
                        bytes_ptr,
                        bytes_len,
                        SQLITE_TRANSIENT(),
                    )
                }
            }

            SqliteArgumentValue::Text(value) => {
                // TODO: Handle text that is too large
                let bytes = value.as_bytes();
                let bytes_ptr = bytes.as_ptr() as *const i8;
                let bytes_len = bytes.len() as i32;

                unsafe {
                    sqlite3_bind_text(
                        statement.handle(),
                        index,
                        bytes_ptr,
                        bytes_len,
                        SQLITE_TRANSIENT(),
                    )
                }
            }

            SqliteArgumentValue::Double(value) => unsafe {
                sqlite3_bind_double(statement.handle(), index, *value)
            },

            SqliteArgumentValue::Int(value) => unsafe {
                sqlite3_bind_int(statement.handle(), index, *value)
            },

            SqliteArgumentValue::Int64(value) => unsafe {
                sqlite3_bind_int64(statement.handle(), index, *value)
            },

            SqliteArgumentValue::Null => unsafe { sqlite3_bind_null(statement.handle(), index) },
        };

        if status != SQLITE_OK {
            return Err(SqliteError::from_connection(statement.connection.0.as_ptr()).into());
        }

        Ok(())
    }
}
