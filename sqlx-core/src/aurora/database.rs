use crate::aurora::arguments::AuroraArguments;
use crate::aurora::column::AuroraColumn;
use crate::aurora::connection::AuroraConnection;
use crate::aurora::done::AuroraDone;
use crate::aurora::row::AuroraRow;
use crate::aurora::statement::AuroraStatement;
use crate::aurora::transaction::AuroraTransactionManager;
use crate::aurora::type_info::AuroraTypeInfo;
use crate::aurora::value::{AuroraValue, AuroraValueRef};
use crate::database::{Database, HasArguments, HasStatement, HasStatementCache, HasValueRef};

use rusoto_rds_data::SqlParameter;

/// Aurora serverless database driver.
#[derive(Debug)]
pub struct Aurora;

impl Database for Aurora {
    type Connection = AuroraConnection;

    type TransactionManager = AuroraTransactionManager;

    type Row = AuroraRow;

    type Done = AuroraDone;

    type Column = AuroraColumn;

    type TypeInfo = AuroraTypeInfo;

    type Value = AuroraValue;
}

impl<'r> HasValueRef<'r> for Aurora {
    type Database = Aurora;

    type ValueRef = AuroraValueRef<'r>;
}

impl HasArguments<'_> for Aurora {
    type Database = Aurora;

    type Arguments = AuroraArguments;

    type ArgumentBuffer = Vec<SqlParameter>;
}

impl<'q> HasStatement<'q> for Aurora {
    type Database = Aurora;

    type Statement = AuroraStatement<'q>;
}

impl HasStatementCache for Aurora {}
