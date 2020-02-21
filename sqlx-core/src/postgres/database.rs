use crate::database::{Database, HasCursor, HasRawValue, HasRow};

/// **Postgres** database driver.
pub struct Postgres;

impl Database for Postgres {
    type Connection = super::PgConnection;

    type Arguments = super::PgArguments;

    type TypeInfo = super::PgTypeInfo;

    type TableId = u32;
}

impl<'a> HasRow<'a> for Postgres {
    // TODO: Can we drop the `type Database = _`
    type Database = Postgres;

    type Row = super::PgRow<'a>;
}

impl<'s, 'q> HasCursor<'s, 'q, Postgres> for Postgres {
    type Cursor = super::PgCursor<'s, 'q>;
}

impl<'a> HasRawValue<'a> for Postgres {
    type RawValue = Option<&'a [u8]>;
}
