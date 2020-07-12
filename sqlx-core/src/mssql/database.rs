use crate::database::{Database, HasArguments, HasValueRef};
use crate::mssql::{
    MssqlArguments, MssqlColumn, MssqlConnection, MssqlDone, MssqlRow, MssqlTransactionManager,
    MssqlTypeInfo, MssqlValue, MssqlValueRef,
};

/// MSSQL database driver.
#[derive(Debug)]
pub struct Mssql;

impl Database for Mssql {
    type Connection = MssqlConnection;

    type TransactionManager = MssqlTransactionManager;

    type Row = MssqlRow;

    type Done = MssqlDone;

    type Column = MssqlColumn;

    type TypeInfo = MssqlTypeInfo;

    type Value = MssqlValue;
}

impl<'r> HasValueRef<'r> for Mssql {
    type Database = Mssql;

    type ValueRef = MssqlValueRef<'r>;
}

impl HasArguments<'_> for Mssql {
    type Database = Mssql;

    type Arguments = MssqlArguments;

    type ArgumentBuffer = Vec<u8>;
}
