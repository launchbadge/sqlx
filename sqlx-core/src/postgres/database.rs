use crate::database::{Database, HasArguments, HasStatementCache, HasValueRef};
use crate::postgres::arguments::PgArgumentBuffer;
use crate::postgres::value::{PgValue, PgValueRef};
use crate::postgres::{
    PgArguments, PgColumn, PgConnection, PgDone, PgRow, PgTransactionManager, PgTypeInfo,
};

/// PostgreSQL database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    type TransactionManager = PgTransactionManager;

    type Row = PgRow;

    type Done = PgDone;

    type Column = PgColumn;

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

    type ArgumentBuffer = PgArgumentBuffer;
}

impl HasStatementCache for Postgres {}
