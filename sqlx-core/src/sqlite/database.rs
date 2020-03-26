use crate::cursor::HasCursor;
use crate::database::Database;
use crate::row::HasRow;
use crate::sqlite::error::SqliteError;
use crate::sqlite::{
    SqliteArgumentValue, SqliteArguments, SqliteConnection, SqliteCursor, SqliteRow,
    SqliteTypeInfo, SqliteValue,
};
use crate::value::HasRawValue;

/// **Sqlite** database driver.
#[derive(Debug)]
pub struct Sqlite;

impl Database for Sqlite {
    type Connection = SqliteConnection;

    type Arguments = SqliteArguments;

    type TypeInfo = SqliteTypeInfo;

    type TableId = String;

    type RawBuffer = Vec<SqliteArgumentValue>;

    type Error = SqliteError;
}

impl<'c> HasRow<'c> for Sqlite {
    type Database = Sqlite;

    type Row = SqliteRow<'c>;
}

impl<'c, 'q> HasCursor<'c, 'q> for Sqlite {
    type Database = Sqlite;

    type Cursor = SqliteCursor<'c, 'q>;
}

impl<'c> HasRawValue<'c> for Sqlite {
    type Database = Sqlite;

    type RawValue = SqliteValue<'c>;
}
