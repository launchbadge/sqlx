use futures_core::future::BoxFuture;
use futures_util::FutureExt;

use crate::error::Error;
use crate::postgres::{PgConnection, Postgres};
use crate::transaction::{
    begin_ansi_transaction, commit_ansi_transaction, rollback_ansi_transaction, TransactionManager,
};

/// Implementation of [`TransactionManager`] for PostgreSQL.
pub struct PgTransactionManager;

impl TransactionManager for PgTransactionManager {
    type Database = Postgres;

    fn begin(conn: &mut PgConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        begin_ansi_transaction(conn, index).boxed()
    }

    fn commit(conn: &mut PgConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        commit_ansi_transaction(conn, index).boxed()
    }

    fn rollback(conn: &mut PgConnection, index: usize) -> BoxFuture<'_, Result<(), Error>> {
        rollback_ansi_transaction(conn, index).boxed()
    }
}
