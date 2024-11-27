use std::borrow::Cow;

use futures_core::future::BoxFuture;

use crate::connection::Waiting;
use crate::error::Error;
use crate::executor::Executor;
use crate::protocol::text::Query;
use crate::{MySql, MySqlConnection};

pub(crate) use sqlx_core::transaction::*;

/// Implementation of [`TransactionManager`] for MySQL.
pub struct MySqlTransactionManager;

impl TransactionManager for MySqlTransactionManager {
    type Database = MySql;

    fn begin<'conn>(
        conn: &'conn mut MySqlConnection,
        statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'conn, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.inner.transaction_depth;
            if statement.is_some() && depth > 0 {
                return Err(Error::InvalidSavePointStatement);
            }
            let statement = statement.unwrap_or_else(|| begin_ansi_transaction_sql(depth));
            conn.execute(&*statement).await?;
            conn.inner.transaction_depth += 1;

            Ok(())
        })
    }

    fn commit(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.inner.transaction_depth;

            if depth > 0 {
                conn.execute(&*commit_ansi_transaction_sql(depth)).await?;
                conn.inner.transaction_depth = depth - 1;
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.inner.transaction_depth;

            if depth > 0 {
                conn.execute(&*rollback_ansi_transaction_sql(depth)).await?;
                conn.inner.transaction_depth = depth - 1;
            }

            Ok(())
        })
    }

    fn start_rollback(conn: &mut MySqlConnection) {
        let depth = conn.inner.transaction_depth;

        if depth > 0 {
            conn.inner.stream.waiting.push_back(Waiting::Result);
            conn.inner.stream.sequence_id = 0;
            conn.inner
                .stream
                .write_packet(Query(&rollback_ansi_transaction_sql(depth)))
                .expect("BUG: unexpected error queueing ROLLBACK");

            conn.inner.transaction_depth = depth - 1;
        }
    }
}
