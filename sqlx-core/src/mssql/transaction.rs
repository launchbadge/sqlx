use std::borrow::Cow;

use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::executor::Executor;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::sql_batch::SqlBatch;
use crate::mssql::{Mssql, MssqlConnection};
use crate::transaction::TransactionManager;

/// Implementation of [`TransactionManager`] for MSSQL.
pub struct MssqlTransactionManager;

impl TransactionManager for MssqlTransactionManager {
    type Database = Mssql;

    fn begin(conn: &mut MssqlConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let query = if depth == 0 {
                Cow::Borrowed("BEGIN TRAN ")
            } else {
                Cow::Owned(format!("SAVE TRAN _sqlx_savepoint_{}", depth))
            };

            conn.execute(&*query).await?;

            Ok(())
        })
    }

    fn commit(conn: &mut MssqlConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if depth == 1 {
                // savepoints are not released in MSSQL
                conn.execute("COMMIT TRAN").await?;
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut MssqlConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let query = if depth == 1 {
                Cow::Borrowed("ROLLBACK TRAN")
            } else {
                Cow::Owned(format!("ROLLBACK TRAN _sqlx_savepoint_{}", depth - 1))
            };

            conn.execute(&*query).await?;

            Ok(())
        })
    }

    fn start_rollback(conn: &mut MssqlConnection, depth: usize) {
        let query = if depth == 1 {
            Cow::Borrowed("ROLLBACK TRAN")
        } else {
            Cow::Owned(format!("ROLLBACK TRAN _sqlx_savepoint_{}", depth - 1))
        };

        conn.pending_done_count += 1;
        conn.stream.write_packet(
            PacketType::SqlBatch,
            SqlBatch {
                transaction_descriptor: conn.stream.transaction_descriptor,
                sql: &*query,
            },
        );
    }
}
