use crate::database::{Database, HasCursor, HasRawValue, HasRow};
use crate::sqlite::error::SqliteError;

/// **Sqlite** database driver.
#[derive(Debug)]
pub struct Sqlite;

impl Database for Sqlite {
    type Connection = super::SqliteConnection;

    type Arguments = super::SqliteArguments;

    type TypeInfo = super::SqliteTypeInfo;

    type TableId = String;

    type RawBuffer = Vec<super::SqliteArgumentValue>;

    type Error = SqliteError;
}

impl<'c> HasRow<'c> for Sqlite {
    type Database = Sqlite;

    type Row = super::SqliteRow<'c>;
}

impl<'c, 'q> HasCursor<'c, 'q> for Sqlite {
    type Database = Sqlite;

    type Cursor = super::SqliteCursor<'c, 'q>;
}

impl<'c> HasRawValue<'c> for Sqlite {
    type RawValue = super::SqliteValue<'c>;
}
