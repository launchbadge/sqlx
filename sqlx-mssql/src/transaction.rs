use sqlx_core::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};

use crate::error::{tiberius_err, Error};
use crate::executor::Executor;
use crate::{Mssql, MssqlConnection};

pub(crate) use sqlx_core::transaction::*;

/// Implementation of [`TransactionManager`] for MSSQL.
///
/// MSSQL uses non-ANSI syntax for savepoints:
/// - depth 0 -> `BEGIN TRANSACTION`
/// - depth N -> `SAVE TRANSACTION _sqlx_savepoint_N`
/// - commit depth 1 -> `COMMIT`
/// - commit depth N -> no-op (savepoints auto-commit with parent)
/// - rollback depth 1 -> `ROLLBACK`
/// - rollback depth N -> `ROLLBACK TRANSACTION _sqlx_savepoint_N`
pub struct MssqlTransactionManager;

impl TransactionManager for MssqlTransactionManager {
    type Database = Mssql;

    async fn begin(conn: &mut MssqlConnection, statement: Option<SqlStr>) -> Result<(), Error> {
        let depth = conn.inner.transaction_depth;

        // Execute any pending rollback first
        resolve_pending_rollback(conn).await?;

        let statement = match statement {
            Some(_) if depth > 0 => return Err(Error::InvalidSavePointStatement),
            Some(statement) => statement,
            None => {
                if depth == 0 {
                    SqlStr::from_static("BEGIN TRANSACTION")
                } else {
                    AssertSqlSafe(format!("SAVE TRANSACTION _sqlx_savepoint_{}", depth))
                        .into_sql_str()
                }
            }
        };

        conn.execute(statement).await?;
        conn.inner.transaction_depth += 1;

        Ok(())
    }

    async fn commit(conn: &mut MssqlConnection) -> Result<(), Error> {
        let depth = conn.inner.transaction_depth;

        if depth > 0 {
            if depth == 1 {
                // Only the outermost transaction actually commits
                conn.execute("COMMIT").await?;
            }
            // Savepoints auto-commit with their parent transaction, so no-op for depth > 1
            conn.inner.transaction_depth = depth - 1;
        }

        Ok(())
    }

    async fn rollback(conn: &mut MssqlConnection) -> Result<(), Error> {
        let depth = conn.inner.transaction_depth;

        if depth > 0 {
            if depth == 1 {
                conn.execute("ROLLBACK").await?;
            } else {
                let savepoint = format!("ROLLBACK TRANSACTION _sqlx_savepoint_{}", depth - 1);
                conn.execute(AssertSqlSafe(savepoint)).await?;
            }
            conn.inner.transaction_depth = depth - 1;
        }

        Ok(())
    }

    fn start_rollback(conn: &mut MssqlConnection) {
        let depth = conn.inner.transaction_depth;
        if depth > 0 {
            // We can't execute async SQL from a synchronous context (Drop),
            // so we set a flag and execute the rollback on the next operation.
            conn.inner.pending_rollback = true;
            conn.inner.transaction_depth = depth - 1;
        }
    }

    fn get_transaction_depth(conn: &MssqlConnection) -> usize {
        conn.inner.transaction_depth
    }
}

/// Execute pending rollback if one was triggered by `start_rollback`.
pub(crate) async fn resolve_pending_rollback(conn: &mut MssqlConnection) -> Result<(), Error> {
    if conn.inner.pending_rollback {
        conn.inner.pending_rollback = false;
        let depth = conn.inner.transaction_depth;
        if depth == 0 {
            // Rollback the entire transaction
            conn.inner
                .client
                .simple_query("ROLLBACK")
                .await
                .map_err(tiberius_err)?
                .into_results()
                .await
                .map_err(tiberius_err)?;
        } else {
            let savepoint = format!("ROLLBACK TRANSACTION _sqlx_savepoint_{}", depth);
            conn.inner
                .client
                .simple_query(savepoint)
                .await
                .map_err(tiberius_err)?
                .into_results()
                .await
                .map_err(tiberius_err)?;
        }
    }
    Ok(())
}
