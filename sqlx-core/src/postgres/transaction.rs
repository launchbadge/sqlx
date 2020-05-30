use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::executor::Executor;
use crate::postgres::message::Query;
use crate::postgres::{PgConnection, Postgres};
use crate::transaction::{
    begin_ansi_transaction_sql, commit_ansi_transaction_sql, rollback_ansi_transaction_sql,
    TransactionManager,
};

/// Implementation of [`TransactionManager`] for PostgreSQL.
pub struct PgTransactionManager;

impl TransactionManager for PgTransactionManager {
    type Database = Postgres;

    fn begin(conn: &mut PgConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            conn.execute(&*begin_ansi_transaction_sql(depth)).await?;

            Ok(())
        })
    }

    fn commit(conn: &mut PgConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            conn.execute(&*commit_ansi_transaction_sql(depth)).await?;

            Ok(())
        })
    }

    fn rollback(conn: &mut PgConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            conn.execute(&*rollback_ansi_transaction_sql(depth)).await?;

            Ok(())
        })
    }

    fn start_rollback(conn: &mut PgConnection, depth: usize) {
        conn.pending_ready_for_query_count += 1;
        conn.stream
            .write(Query(&rollback_ansi_transaction_sql(depth)));
    }
}
