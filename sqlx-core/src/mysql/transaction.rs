use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::executor::Executor;
use crate::mysql::connection::Waiting;
use crate::mysql::protocol::text::Query;
use crate::mysql::{MySql, MySqlConnection};
use crate::transaction::{
    begin_savepoint_sql, commit_savepoint_sql, rollback_savepoint_sql, TransactionManager,
    COMMIT_ANSI_TRANSACTION, ROLLBACK_ANSI_TRANSACTION,
};

/// Implementation of [`TransactionManager`] for MySQL.
pub struct MySqlTransactionManager;

impl TransactionManager for MySqlTransactionManager {
    type Database = MySql;
    type Options = MySqlTransactionOptions;

    fn begin_with(
        conn: &mut MySqlConnection,
        options: MySqlTransactionOptions,
    ) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            let stmt = if depth == 0 {
                options.sql()
            } else {
                begin_savepoint_sql(depth)
            };
            conn.execute(&*stmt).await?;
            conn.transaction_depth = depth + 1;

            Ok(())
        })
    }

    fn commit(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
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

    fn rollback(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
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

    fn start_rollback(conn: &mut MySqlConnection) {
        let depth = conn.transaction_depth;
        if depth > 0 {
            conn.stream.waiting.push_back(Waiting::Result);
            conn.stream.sequence_id = 0;
            if depth == 1 {
                conn.stream.write_packet(Query(ROLLBACK_ANSI_TRANSACTION));
            } else {
                conn.stream
                    .write_packet(Query(&rollback_savepoint_sql(depth)));
            }
            conn.transaction_depth -= 1;
        }
    }
}

/// Transaction initiation options for MySQL.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MySqlTransactionOptions {
    pub(crate) consistent_read: bool,
    pub(crate) read_only: bool,
    pub(crate) iso_level: Option<MySqlIsolationLevel>,
}

impl MySqlTransactionOptions {
    pub fn consistent_read(mut self) -> Self {
        self.consistent_read = true;
        self
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn isolation_level(mut self, level: MySqlIsolationLevel) -> Self {
        self.iso_level.replace(level);
        self
    }

    pub(crate) fn sql(&self) -> String {
        let mut sql = String::with_capacity(64);
        if let Some(level) = self.iso_level {
            sql.push_str("SET TRANSACTION");
            match level {
                MySqlIsolationLevel::Serializable => sql.push_str(" ISOLATION LEVEL SERIALIZABLE"),
                MySqlIsolationLevel::RepeatableRead => {
                    sql.push_str(" ISOLATION LEVEL REPEATABLE READ")
                }
                MySqlIsolationLevel::ReadCommitted => {
                    sql.push_str(" ISOLATION LEVEL READ COMMITTED")
                }
                MySqlIsolationLevel::ReadUncommitted => {
                    sql.push_str(" ISOLATION LEVEL READ UNCOMMITTED")
                }
            }
            sql.push_str("; ");
        }
        sql.push_str("START TRANSACTION");
        if self.read_only {
            sql.push_str(" READ ONLY");
        }
        if self.consistent_read {
            sql.push_str(" WITH CONSISTENT SNAPSHOT");
        }
        sql
    }
}

impl From<MySqlIsolationLevel> for MySqlTransactionOptions {
    fn from(iso_level: MySqlIsolationLevel) -> Self {
        Self {
            iso_level: Some(iso_level),
            ..Default::default()
        }
    }
}

/// Transaction isolation levels for MySQL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MySqlIsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl Default for MySqlIsolationLevel {
    fn default() -> Self {
        Self::RepeatableRead
    }
}
