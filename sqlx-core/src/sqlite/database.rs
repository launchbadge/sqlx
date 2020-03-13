use crate::database::{Database, HasCursor, HasRawValue, HasRow};

/// **Sqlite** database driver.
pub struct Sqlite;

impl Database for Sqlite {
    type Connection = super::SqliteConnection;

    type Arguments = super::SqliteArguments;

    type TypeInfo = super::SqliteTypeInfo;

    // TODO?
    type TableId = u32;

    type RawBuffer = Vec<super::SqliteArgumentValue>;
}

impl<'c> HasRow<'c> for Sqlite {
    type Database = Sqlite;

    type Row = super::SqliteRow<'c>;
}

impl<'s, 'q> HasCursor<'s, 'q> for Sqlite {
    type Database = Sqlite;

    type Cursor = super::SqliteCursor<'s, 'q>;
}

impl<'c> HasRawValue<'c> for Sqlite {
    type RawValue = super::SqliteResultValue<'c>;
}
