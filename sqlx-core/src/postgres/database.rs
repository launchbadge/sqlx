use crate::database::{Database, HasArguments, HasRawValue};
use crate::postgres::{PgArguments, PgConnection, PgRawBuffer, PgRawValue, PgRow, PgTypeInfo};

/// PostgreSQL database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    type Row = PgRow;

    type RawBuffer = PgRawBuffer;

    type TypeInfo = PgTypeInfo;
}

impl<'r> HasRawValue<'r> for Postgres {
    type Database = Postgres;

    type RawValue = PgRawValue<'r>;
}

impl HasArguments<'_> for Postgres {
    type Database = Postgres;

    type Arguments = PgArguments;
}
