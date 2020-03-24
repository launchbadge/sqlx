use crate::row::{ColumnIndex, Row};
use crate::sqlite::statement::Statement;
use crate::sqlite::value::SqliteValue;
use crate::sqlite::{Sqlite, SqliteConnection};

pub struct SqliteRow<'c> {
    pub(super) values: usize,
    pub(super) statement: Option<usize>,
    pub(super) connection: &'c SqliteConnection,
}

// Accessing values from the statement object is
// safe across threads as long as we don't call [sqlite3_step]
// That should not be possible as long as an immutable borrow is held on the connection

#[allow(unsafe_code)]
unsafe impl Send for SqliteRow<'_> {}

impl<'c> SqliteRow<'c> {
    #[inline]
    fn statement(&self) -> &'c Statement {
        self.connection.statement(self.statement)
    }
}

impl<'c> Row<'c> for SqliteRow<'c> {
    type Database = Sqlite;

    #[inline]
    fn len(&self) -> usize {
        self.values
    }

    fn try_get_raw<I>(&self, index: I) -> crate::Result<Sqlite, SqliteValue<'c>>
    where
        I: ColumnIndex<'c, Self>,
    {
        Ok(SqliteValue {
            statement: self.statement(),
            index: index.resolve(self)? as i32,
        })
    }
}

impl<'c> ColumnIndex<'c, SqliteRow<'c>> for usize {
    fn resolve(self, row: &SqliteRow<'c>) -> crate::Result<Sqlite, usize> {
        let len = Row::len(row);

        if self >= len {
            return Err(crate::Error::ColumnIndexOutOfBounds { len, index: self });
        }

        Ok(self)
    }
}

impl<'c> ColumnIndex<'c, SqliteRow<'c>> for &'c str {
    fn resolve(self, row: &SqliteRow<'c>) -> crate::Result<Sqlite, usize> {
        row.statement()
            .columns
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
            .map(|&index| index as usize)
    }
}
