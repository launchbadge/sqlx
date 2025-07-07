use std::future::Future;

use sqlx_core::transaction::TransactionManager;
use sqlx_core::{error::Error, sql_str::SqlStr};

use crate::{Sqlite, SqliteConnection};

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    async fn begin(conn: &mut SqliteConnection, statement: Option<SqlStr>) -> Result<(), Error> {
        let is_custom_statement = statement.is_some();
        conn.worker.begin(statement).await?;
        if is_custom_statement {
            // Check that custom statement actually put the connection into a transaction.
            let mut handle = conn.lock_handle().await?;
            if !handle.in_transaction() {
                return Err(Error::BeginFailed);
            }
        }
        Ok(())
    }

    fn commit(conn: &mut SqliteConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.worker.commit()
    }

    fn rollback(
        conn: &mut SqliteConnection,
    ) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.worker.rollback()
    }

    fn start_rollback(conn: &mut SqliteConnection) {
        conn.worker.start_rollback().ok();
    }

    fn get_transaction_depth(conn: &SqliteConnection) -> usize {
        conn.worker.shared.get_transaction_depth()
    }
}
