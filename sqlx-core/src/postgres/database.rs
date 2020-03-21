//! Types which represent various database drivers.

use crate::database::{Database, HasCursor, HasRawValue, HasRow};
use crate::postgres::error::PgError;
use crate::postgres::row::PgValue;

/// **Postgres** database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = super::PgConnection;

    type Arguments = super::PgArguments;

    type TypeInfo = super::PgTypeInfo;

    type TableId = u32;

    type RawBuffer = Vec<u8>;

    type Error = PgError;
}

impl<'a> HasRow<'a> for Postgres {
    type Database = Postgres;

    type Row = super::PgRow<'a>;
}

impl<'s, 'q> HasCursor<'s, 'q> for Postgres {
    type Database = Postgres;

    type Cursor = super::PgCursor<'s, 'q>;
}

impl<'a> HasRawValue<'a> for Postgres {
    type RawValue = Option<PgValue<'a>>;
}
