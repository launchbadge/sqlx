use crate::database::HasRow;
use crate::row::{ColumnIndex, Row};
use crate::sqlite::statement::Statement;
use crate::sqlite::value::SqliteValue;
use crate::sqlite::{Sqlite, SqliteConnection};

pub struct SqliteRow<'c> {
    pub(super) values: usize,
    pub(super) statement: Option<usize>,
    pub(super) connection: &'c mut SqliteConnection,
}

impl<'c> SqliteRow<'c> {
    fn statement(&'c self) -> &'c Statement {
        self.connection.statement(self.statement)
    }
}

impl<'c> Row<'c> for SqliteRow<'c> {
    type Database = Sqlite;

    #[inline]
    fn len(&self) -> usize {
        self.values
    }

    fn try_get_raw<'r, I>(&'r self, index: I) -> crate::Result<Sqlite, SqliteValue<'r>>
    where
        'c: 'r,
        I: ColumnIndex<Self::Database>,
    {
        let index = index.resolve(self)?;
        let value = SqliteValue::new(self.statement(), index);

        Ok(value)
    }
}

impl ColumnIndex<Sqlite> for usize {
    fn resolve(self, row: &<Sqlite as HasRow>::Row) -> crate::Result<Sqlite, usize> {
        let len = Row::len(row);

        if self >= len {
            return Err(crate::Error::ColumnIndexOutOfBounds { len, index: self });
        }

        Ok(self)
    }
}

impl ColumnIndex<Sqlite> for &'_ str {
    fn resolve(self, row: &<Sqlite as HasRow>::Row) -> crate::Result<Sqlite, usize> {
        row.statement()
            .columns
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
            .map(|&index| index as usize)
    }
}
