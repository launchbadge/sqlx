use crate::connection::Waiting;
use crate::error::Error;
use crate::executor::Executor;
use crate::protocol::text::Query;
use crate::{MySql, MySqlConnection};
use futures_core::future::BoxFuture;
use sqlx_core::database::Database;
use std::borrow::Cow;

pub(crate) use sqlx_core::transaction::*;

/// Implementation of [`TransactionManager`] for MySQL.
pub struct MySqlTransactionManager;

impl TransactionManager for MySqlTransactionManager {
    type Database = MySql;

    fn begin(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;

            conn.execute(&*begin_ansi_transaction_sql(depth)).await?;
            conn.transaction_depth = depth + 1;

            Ok(())
        })
    }

    fn begin_with<'a, S>(
        conn: &'a mut <Self::Database as Database>::Connection,
        sql: S,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        S: Into<Cow<'static, str>> + Send + 'a,
    {
        Box::pin(async move {
            let depth = conn.transaction_depth;

            conn.execute(&*sql.into()).await?;
            conn.transaction_depth = depth + 1;

            Ok(())
        })
    }

    fn commit(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;

            if depth > 0 {
                conn.execute(&*commit_ansi_transaction_sql(depth)).await?;
                conn.transaction_depth = depth - 1;
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut MySqlConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;

            if depth > 0 {
                conn.execute(&*rollback_ansi_transaction_sql(depth)).await?;
                conn.transaction_depth = depth - 1;
            }

            Ok(())
        })
    }

    fn start_rollback(conn: &mut MySqlConnection) {
        let depth = conn.transaction_depth;

        if depth > 0 {
            conn.stream.waiting.push_back(Waiting::Result);
            conn.stream.sequence_id = 0;
            conn.stream
                .write_packet(Query(&*rollback_ansi_transaction_sql(depth)));

            conn.transaction_depth = depth - 1;
        }
    }
}
