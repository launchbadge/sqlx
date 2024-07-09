use crate::{Sqlite, SqliteConnection};
use futures_core::future::BoxFuture;
use sqlx_core::database::Database;
use sqlx_core::error::Error;
use sqlx_core::transaction::TransactionManager;
use std::borrow::Cow;

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    fn begin(conn: &mut SqliteConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(conn.worker.begin())
    }

    fn begin_with<'a, S>(
        _conn: &'a mut <Self::Database as Database>::Connection,
        sql: S,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        S: Into<Cow<'static, str>> + Send + 'a,
    {
        unimplemented!()
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
