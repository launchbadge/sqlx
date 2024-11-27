use futures_core::future::BoxFuture;
use std::borrow::Cow;

use crate::{Sqlite, SqliteConnection};
use sqlx_core::error::Error;
use sqlx_core::transaction::TransactionManager;

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    fn begin<'conn>(
        conn: &'conn mut SqliteConnection,
        statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'conn, Result<(), Error>> {
        Box::pin(conn.worker.begin(statement))
    }

    fn commit(conn: &mut SqliteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(conn.worker.commit())
    }

    fn rollback(conn: &mut SqliteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(conn.worker.rollback())
    }

    fn start_rollback(conn: &mut SqliteConnection) {
        conn.worker.start_rollback().ok();
    }
}
