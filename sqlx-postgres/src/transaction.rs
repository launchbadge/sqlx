use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::executor::Executor;

use crate::{PgConnection, Postgres};

pub(crate) use sqlx_core::transaction::*;

/// Implementation of [`TransactionManager`] for PostgreSQL.
pub struct PgTransactionManager;

impl TransactionManager for PgTransactionManager {
    type Database = Postgres;

    fn begin(conn: &mut PgConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let rollback = Rollback::new(conn);
            let query = begin_ansi_transaction_sql(rollback.conn.transaction_depth);
            rollback.conn.queue_simple_query(&query);
            rollback.conn.transaction_depth += 1;
            rollback.conn.wait_until_ready().await?;
            rollback.defuse();

            Ok(())
        })
    }

    fn commit(conn: &mut PgConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if conn.transaction_depth > 0 {
                conn.execute(&*commit_ansi_transaction_sql(conn.transaction_depth))
                    .await?;

                conn.transaction_depth -= 1;
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut PgConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if conn.transaction_depth > 0 {
                conn.execute(&*rollback_ansi_transaction_sql(conn.transaction_depth))
                    .await?;

                conn.transaction_depth -= 1;
            }

            Ok(())
        })
    }

    fn start_rollback(conn: &mut PgConnection) {
        if conn.transaction_depth > 0 {
            conn.queue_simple_query(&rollback_ansi_transaction_sql(conn.transaction_depth));

            conn.transaction_depth -= 1;
        }
    }
}

struct Rollback<'c> {
    conn: &'c mut PgConnection,
    defuse: bool,
}

impl Drop for Rollback<'_> {
    fn drop(&mut self) {
        if !self.defuse {
            PgTransactionManager::start_rollback(self.conn)
        }
    }
}

impl<'c> Rollback<'c> {
    fn new(conn: &'c mut PgConnection) -> Self {
        Self {
            conn,
            defuse: false,
        }
    }
    fn defuse(mut self) {
        self.defuse = true;
    }
}
