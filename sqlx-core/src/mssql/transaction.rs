use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::executor::Executor;
use crate::mssql::{MsSql, MsSqlConnection};
use crate::transaction::{
    begin_ansi_transaction_sql, commit_ansi_transaction_sql, rollback_ansi_transaction_sql,
    TransactionManager,
};

/// Implementation of [`TransactionManager`] for MSSQL.
pub struct MsSqlTransactionManager;

impl TransactionManager for MsSqlTransactionManager {
    type Database = MsSql;

    fn begin(conn: &mut MsSqlConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    fn commit(conn: &mut MsSqlConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    fn rollback(conn: &mut MsSqlConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    fn start_rollback(conn: &mut MsSqlConnection, depth: usize) {
        unimplemented!()
    }
}
