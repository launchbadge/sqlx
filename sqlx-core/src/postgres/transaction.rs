use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::executor::Executor;
use crate::postgres::{PgConnection, Postgres};
use crate::transaction::{
    begin_savepoint_sql, commit_savepoint_sql, rollback_savepoint_sql, TransactionManager,
    COMMIT_ANSI_TRANSACTION, ROLLBACK_ANSI_TRANSACTION,
};

/// Implementation of [`TransactionManager`] for PostgreSQL.
pub struct PgTransactionManager;

impl TransactionManager for PgTransactionManager {
    type Database = Postgres;
    type Options = PgTransactionOptions;

    fn begin_with(
        conn: &mut PgConnection,
        options: PgTransactionOptions,
    ) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            let stmt = if depth == 0 {
                options.sql()
            } else {
                begin_savepoint_sql(depth)
            };
            conn.execute(&*stmt).await?;
            conn.transaction_depth += 1;

            Ok(())
        })
    }

    fn commit(conn: &mut PgConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                let stmt = if depth == 1 {
                    COMMIT_ANSI_TRANSACTION.to_string()
                } else {
                    commit_savepoint_sql(depth)
                };
                conn.execute(&*stmt).await?;
                conn.transaction_depth -= 1;
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut PgConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                let stmt = if depth == 1 {
                    ROLLBACK_ANSI_TRANSACTION.to_string()
                } else {
                    rollback_savepoint_sql(depth)
                };
                conn.execute(&*stmt).await?;
                conn.transaction_depth -= 1;
            }

            Ok(())
        })
    }

    fn start_rollback(conn: &mut PgConnection) {
        let depth = conn.transaction_depth;
        if depth > 0 {
            if depth == 1 {
                conn.queue_simple_query(ROLLBACK_ANSI_TRANSACTION)
            } else {
                conn.queue_simple_query(&rollback_savepoint_sql(depth))
            }
            conn.transaction_depth -= 1;
        }
    }
}

/// Transaction initiation options for Postgres.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PgTransactionOptions {
    pub(crate) deferrable: bool,
    pub(crate) read_only: bool,
    pub(crate) iso_level: Option<PgIsolationLevel>,
}

impl PgTransactionOptions {
    pub fn deferrable(mut self) -> Self {
        self.deferrable = true;
        self
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn isolation_level(mut self, level: PgIsolationLevel) -> Self {
        self.iso_level.replace(level);
        self
    }

    pub(crate) fn sql(&self) -> String {
        let mut sql = String::with_capacity(64);
        sql.push_str("BEGIN");
        match self.iso_level {
            None => (),
            Some(PgIsolationLevel::ReadUncommitted) => {
                sql.push_str(" ISOLATION LEVEL READ UNCOMMITTED")
            }
            Some(PgIsolationLevel::ReadCommitted) => {
                sql.push_str(" ISOLATION LEVEL READ COMMITTED")
            }
            Some(PgIsolationLevel::RepeatableRead) => {
                sql.push_str(" ISOLATION LEVEL REPEATABLE READ")
            }
            Some(PgIsolationLevel::Serializable) => sql.push_str(" ISOLATION LEVEL SERIALIZABLE"),
        }
        if self.read_only {
            sql.push_str(" READ ONLY");
        }
        if self.deferrable {
            sql.push_str(" DEFERRABLE");
        }
        sql
    }
}

impl From<PgIsolationLevel> for PgTransactionOptions {
    fn from(iso_level: PgIsolationLevel) -> Self {
        Self {
            iso_level: Some(iso_level),
            ..Default::default()
        }
    }
}

/// Transaction isolation levels for Postgres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PgIsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl Default for PgIsolationLevel {
    fn default() -> Self {
        Self::ReadCommitted
    }
}
