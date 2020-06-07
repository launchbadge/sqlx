use crate::database::{Database, HasArguments, HasValueRef};
use crate::mssql::{
    MsSqlArguments, MsSqlConnection, MsSqlRow, MsSqlTransactionManager, MsSqlTypeInfo, MsSqlValue,
    MsSqlValueRef,
};

/// MSSQL database driver.
#[derive(Debug)]
pub struct MsSql;

impl Database for MsSql {
    type Connection = MsSqlConnection;

    type TransactionManager = MsSqlTransactionManager;

    type Row = MsSqlRow;

    type TypeInfo = MsSqlTypeInfo;

    type Value = MsSqlValue;
}

impl<'r> HasValueRef<'r> for MsSql {
    type Database = MsSql;

    type ValueRef = MsSqlValueRef<'r>;
}

impl HasArguments<'_> for MsSql {
    type Database = MsSql;

    type Arguments = MsSqlArguments;

    type ArgumentBuffer = Vec<u8>;
}
