use crate::database::{Database, HasArguments, HasValueRef};
use crate::mysql::value::{MySqlValue, MySqlValueRef};
use crate::mysql::{
    MySqlArguments, MySqlConnection, MySqlRow, MySqlTransactionManager, MySqlTypeInfo,
};

/// MySQL database driver.
#[derive(Debug)]
pub struct MySql;

impl Database for MySql {
    type Connection = MySqlConnection;

    type TransactionManager = MySqlTransactionManager;

    type Row = MySqlRow;

    type TypeInfo = MySqlTypeInfo;

    type Value = MySqlValue;
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
