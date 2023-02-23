use crate::arguments::PgArgumentBuffer;
use crate::value::{PgValue, PgValueRef};
use crate::{
    PgArguments, PgColumn, PgConnection, PgQueryResult, PgRow, PgStatement, PgTransactionManager,
    PgTypeInfo,
};

pub(crate) use sqlx_core::database::{
    Database, HasArguments, HasStatement, HasStatementCache, HasValueRef,
};

/// PostgreSQL database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    type TransactionManager = PgTransactionManager;

    type Row = PgRow;

    type QueryResult = PgQueryResult;

    type Column = PgColumn;

    type TypeInfo = PgTypeInfo;

    type Value = PgValue;

    const NAME: &'static str = "PostgreSQL";

    const URL_SCHEMES: &'static [&'static str] = &["postgres", "postgresql"];
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

impl<'q> HasStatement<'q> for Postgres {
    type Database = Postgres;

    type Statement = PgStatement<'q>;
}

impl HasStatementCache for Postgres {}
