use crate::database::{Database, HasArguments, HasValueRef};
use crate::postgres::value::{PgValue, PgValueRef};
use crate::postgres::{PgArguments, PgConnection, PgRow, PgTypeInfo};

/// PostgreSQL database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    type Row = PgRow;

    type TypeInfo = PgTypeInfo;

    type Value = PgValue;
}

impl<'r> HasValueRef<'r> for Postgres {
    type Database = Postgres;

    type ValueRef = PgValueRef<'r>;
}

impl HasArguments<'_> for Postgres {
    type Database = Postgres;

    type Arguments = PgArguments;
}
