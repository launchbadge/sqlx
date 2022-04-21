use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::sqlite::{Sqlite, SqliteConnection};
use crate::transaction::TransactionManager;

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;
    type Options = SqliteTransactionOptions;

    fn begin_with(
        conn: &mut SqliteConnection,
        options: SqliteTransactionOptions,
    ) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(conn.worker.begin(options))
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

/// Transaction initiation options for SQLite.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SqliteTransactionOptions {
    pub(crate) behavior: SqliteTransactionBehavior,
}

impl SqliteTransactionOptions {
    pub fn behavior(mut self, behavior: SqliteTransactionBehavior) -> Self {
        self.behavior = behavior;
        self
    }
}

impl From<SqliteTransactionBehavior> for SqliteTransactionOptions {
    fn from(behavior: SqliteTransactionBehavior) -> Self {
        Self { behavior }
    }
}

/// Transaction behaviors for SQLite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqliteTransactionBehavior {
    Deferred,
    Immediate,
    Exclusive,
}

impl SqliteTransactionBehavior {
    pub(crate) fn sql(&self) -> &'static str {
        match self {
            Self::Deferred => "BEGIN DEFERRED",
            Self::Immediate => "BEGIN IMMEDIATE",
            Self::Exclusive => "BEGIN EXCLUSIVE",
        }
    }
}

impl Default for SqliteTransactionBehavior {
    fn default() -> Self {
        Self::Deferred
    }
}

/// Transaction state enum for a SQLite connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqliteTransactionState {
    None,
    Read,
    Write,
}
