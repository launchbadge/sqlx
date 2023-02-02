use crate::value::{MySqlValue, MySqlValueRef};
use crate::{
    MySqlArguments, MySqlColumn, MySqlConnection, MySqlQueryResult, MySqlRow, MySqlStatement,
    MySqlTransactionManager, MySqlTypeInfo,
};
pub(crate) use sqlx_core::database::{
    Database, HasArguments, HasStatement, HasStatementCache, HasValueRef,
};

/// MySQL database driver.
#[derive(Debug)]
pub struct MySql;

impl Database for MySql {
    type Connection = MySqlConnection;

    type TransactionManager = MySqlTransactionManager;

    type Row = MySqlRow;

    type QueryResult = MySqlQueryResult;

    type Column = MySqlColumn;

    type TypeInfo = MySqlTypeInfo;

    type Value = MySqlValue;

    const NAME: &'static str = "MySQL";

    const URL_SCHEMES: &'static [&'static str] = &["mysql", "mariadb"];
}

impl<'r> HasValueRef<'r> for MySql {
    type Database = MySql;

    type ValueRef = MySqlValueRef<'r>;
}

impl HasArguments<'_> for MySql {
    type Database = MySql;

    type Arguments = MySqlArguments;

    type ArgumentBuffer = Vec<u8>;
}

impl<'q> HasStatement<'q> for MySql {
    type Database = MySql;

    type Statement = MySqlStatement<'q>;
}

impl HasStatementCache for MySql {}
