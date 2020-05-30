use futures_core::future::BoxFuture;
use futures_util::FutureExt;

use crate::error::Error;
use crate::sqlite::{Sqlite, SqliteConnection};
use crate::transaction::{
    begin_ansi_transaction, commit_ansi_transaction, rollback_ansi_transaction, TransactionManager,
};

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    fn begin(conn: &mut SqliteConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        begin_ansi_transaction(conn, index).boxed()
    }

    fn commit(conn: &mut SqliteConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        commit_ansi_transaction(conn, index).boxed()
    }

    fn rollback(conn: &mut SqliteConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        rollback_ansi_transaction(conn, index).boxed()
    }
}
