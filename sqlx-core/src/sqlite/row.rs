use libsqlite3_sys::sqlite3_data_count;
use std::os::raw::c_int;

use crate::database::HasRow;
use crate::row::{ColumnIndex, Row};
use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::value::SqliteResultValue;
use crate::sqlite::{Sqlite, SqliteConnection};

pub struct SqliteRow<'c> {
    pub(super) statement: Option<usize>,
    pub(super) connection: &'c SqliteConnection,
}

impl SqliteRow<'_> {
    fn statement(&self) -> &SqliteStatement {
        self.connection.statement(self.statement)
    }
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
        let count: c_int = unsafe { sqlite3_data_count(self.statement().handle()) };

        count as usize
    }

    fn try_get_raw<'r, I>(&'r self, index: I) -> crate::Result<SqliteResultValue<'c>>
    where
        I: ColumnIndex<Self::Database>,
    {
        let index = index.resolve(self)?;
        let value = SqliteResultValue {
            index,
            connection: self.connection,
            statement: self.statement,
        };

        Ok(value)
    }
}

impl ColumnIndex<Sqlite> for usize {
    fn resolve(self, row: &<Sqlite as HasRow>::Row) -> crate::Result<usize> {
        let len = Row::len(row);

        if self >= len {
            return Err(crate::Error::ColumnIndexOutOfBounds { len, index: self });
        }

        Ok(self)
    }
}

impl ColumnIndex<Sqlite> for &'_ str {
    fn resolve(self, row: &<Sqlite as HasRow>::Row) -> crate::Result<usize> {
        row.statement()
            .columns
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
            .map(|&index| index as usize)
    }
}
