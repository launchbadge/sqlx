//! Types which represent various database drivers.

use crate::cursor::HasCursor;
use crate::database::Database;
use crate::postgres::{PgArguments, PgConnection, PgCursor, PgError, PgRow, PgTypeInfo, PgValue};
use crate::row::HasRow;
use crate::value::HasRawValue;

/// **Postgres** database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    type Arguments = PgArguments;

    type TypeInfo = PgTypeInfo;

    type TableId = u32;

    type RawBuffer = Vec<u8>;

    type Error = PgError;
}

impl<'a> HasRow<'a> for Postgres {
    type Database = Postgres;

    type Row = PgRow<'a>;
}

impl<'s, 'q> HasCursor<'s, 'q> for Postgres {
    type Database = Postgres;

    type Cursor = PgCursor<'s, 'q>;
}

impl<'a> HasRawValue<'a> for Postgres {
    type Database = Postgres;

    type RawValue = PgValue<'a>;
}
