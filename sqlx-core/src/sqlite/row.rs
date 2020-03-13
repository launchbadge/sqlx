use std::collections::HashMap;
use std::sync::Arc;

use libc::c_int;
use libsqlite3_sys::sqlite3_data_count;

use crate::row::{ColumnIndex, Row};
use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::value::SqliteResultValue;
use crate::sqlite::Sqlite;

pub struct SqliteRow<'c> {
    pub(super) statement: &'c SqliteStatement,
    // TODO
    pub(super) columns: Arc<HashMap<Box<str>, u16>>,
}

impl<'c> Row<'c> for SqliteRow<'c> {
    type Database = Sqlite;

    fn len(&self) -> usize {
        // https://sqlite.org/c3ref/data_count.html

        // The sqlite3_data_count(P) interface returns the number of columns
        // in the current row of the result set.

        // The value is correct only if there was a recent call to
        // sqlite3_step that returned SQLITE_ROW.

        #[allow(unsafe_code)]
        let count: c_int = unsafe { sqlite3_data_count(self.statement.handle.as_ptr()) };

        count as usize
    }

    fn try_get_raw<'r, I>(&'r self, index: I) -> crate::Result<SqliteResultValue<'c>>
    where
        I: ColumnIndex<Self::Database>,
    {
        let index = index.resolve(self)?;
        let value = SqliteResultValue {
            index,
            statement: self.statement,
        };

        Ok(value)
    }
}
