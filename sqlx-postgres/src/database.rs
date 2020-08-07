use crate::{PgConnection, PgTypeId, PgTypeInfo};
use sqlx_core::database::{Database, HasStatementCache, HasTypeId};

/// PostgreSQL database driver.
#[derive(Debug)]
pub struct Postgres;

impl Database for Postgres {
    type Connection = PgConnection;

    // type TransactionManager = PgTransactionManager;
    //
    // type Row = PgRow;
    //
    // type Done = PgDone;
    //
    // type Column = PgColumn;
    //
    type TypeInfo = PgTypeInfo;
    //
    // type Value = PgValue;
}

impl<'a> HasTypeId<'a> for Postgres {
    type Database = Postgres;

    type TypeId = PgTypeId<'a>;
}

// impl<'r> HasValueRef<'r> for Postgres {
//     type Database = Postgres;
//
//     type ValueRef = PgValueRef<'r>;
// }
//
// impl HasArguments<'_> for Postgres {
//     type Database = Postgres;
//
//     type Arguments = PgArguments;
//
//     type ArgumentBuffer = PgArgumentBuffer;
// }
//
// impl<'q> HasStatement<'q> for Postgres {
//     type Database = Postgres;
//
//     type Statement = PgStatement<'q>;
// }

impl HasStatementCache for Postgres {}
