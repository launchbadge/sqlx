use crate::{Sqlite, SqliteConnection};
use sqlx_core::error::Error;
use sqlx_core::transaction::TransactionManager;

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    async fn begin(conn: &mut SqliteConnection) -> Result<(), Error> {
        conn.worker.begin().await
    }

    async fn commit(conn: &mut SqliteConnection) -> Result<(), Error> {
        conn.worker.commit().await
    }

    async fn rollback(conn: &mut SqliteConnection) -> Result<(), Error> {
        conn.worker.rollback().await
    }

    fn start_rollback(conn: &mut SqliteConnection) {
        conn.worker.start_rollback().ok();
    }
}
