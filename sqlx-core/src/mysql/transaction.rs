use futures_core::future::BoxFuture;
use futures_util::FutureExt;

use crate::error::Error;
use crate::mysql::{MySql, MySqlConnection};
use crate::transaction::{
    begin_ansi_transaction, commit_ansi_transaction, rollback_ansi_transaction, TransactionManager,
};

/// Implementation of [`TransactionManager`] for MySQL.
pub struct MySqlTransactionManager;

impl TransactionManager for MySqlTransactionManager {
    type Database = MySql;

    fn begin(conn: &mut MySqlConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        begin_ansi_transaction(conn, index).boxed()
    }

    fn commit(conn: &mut MySqlConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        commit_ansi_transaction(conn, index).boxed()
    }

    fn rollback(conn: &mut MySqlConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        rollback_ansi_transaction(conn, index).boxed()
    }
}
