use crate::row::{Column, ColumnIndex, Row};
use crate::sqlite::statement::Statement;
use crate::sqlite::value::SqliteValue;
use crate::sqlite::{Sqlite, SqliteConnection};

pub struct SqliteRow<'c> {
    pub(super) values: usize,
    pub(super) statement: Option<usize>,
    pub(super) connection: &'c SqliteConnection,
}

impl crate::row::private_row::Sealed for SqliteRow<'_> {}

// Accessing values from the statement object is
// safe across threads as long as we don't call [sqlite3_step]
// That should not be possible as long as an immutable borrow is held on the connection

unsafe impl Send for SqliteRow<'_> {}
unsafe impl Sync for SqliteRow<'_> {}

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

    #[doc(hidden)]
    fn try_get_raw<I>(&self, index: I) -> crate::Result<SqliteValue<'c>>
    where
        I: ColumnIndex<'c, Self>,
    {
        Ok(SqliteValue {
            statement: self.statement(),
            index: index.index(self)? as i32,
        })
    }

    fn columns(&self) -> Box<[Column<Self::Database>]> {
        let statement = self.statement();
        let mut sorted_column_names: Vec<(&String, &usize)> = statement.columns.iter().collect();
        sorted_column_names.sort_by_key(|(_, index)| *index);

        let mut columns = Vec::with_capacity(statement.columns.len());
        for (column_name, _index) in sorted_column_names {
            columns.push(Column {
                name: Some(column_name),
                type_info: None,
            });
        }
        columns.into_boxed_slice()
    }
}

impl<'c> ColumnIndex<'c, SqliteRow<'c>> for usize {
    fn index(&self, row: &SqliteRow<'c>) -> crate::Result<usize> {
        let len = Row::len(row);

        if *self >= len {
            return Err(crate::Error::ColumnIndexOutOfBounds { len, index: *self });
        }

        Ok(*self)
    }
}

impl<'c> ColumnIndex<'c, SqliteRow<'c>> for str {
    fn index(&self, row: &SqliteRow<'c>) -> crate::Result<usize> {
        row.statement()
            .columns
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
            .map(|&index| index as usize)
    }
}
