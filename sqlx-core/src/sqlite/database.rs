use crate::database::{Database, HasCursor, HasRawValue, HasRow};
use crate::sqlite::arguments::SqliteValue;

/// **Sqlite** database driver.
pub struct Sqlite;

impl Database for Sqlite {
    type Connection = super::SqliteConnection;

    type Arguments = super::SqliteArguments;

    type TypeInfo = super::SqliteTypeInfo;

    // TODO?
    type TableId = u32;

    type RawBuffer = Vec<SqliteValue>;
}

impl<'a> HasRow<'a> for Sqlite {
    type Database = Sqlite;

    type Row = super::SqliteRow<'a>;
}

impl<'s, 'q> HasCursor<'s, 'q> for Sqlite {
    type Database = Sqlite;

    type Cursor = super::SqliteCursor<'s, 'q>;
}

impl<'a> HasRawValue<'a> for Sqlite {
    // TODO
    type RawValue = Option<()>;
}
