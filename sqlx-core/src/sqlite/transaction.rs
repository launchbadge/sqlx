use std::ptr;

use futures_core::future::BoxFuture;
use libsqlite3_sys::{sqlite3_exec, SQLITE_OK};

use crate::error::Error;
use crate::executor::Executor;
use crate::sqlite::{Sqlite, SqliteConnection, SqliteError};
use crate::transaction::{
    begin_ansi_transaction_sql, commit_ansi_transaction_sql, rollback_ansi_transaction_sql,
    TransactionManager,
};

/// Implementation of [`TransactionManager`] for SQLite.
pub struct SqliteTransactionManager;

impl TransactionManager for SqliteTransactionManager {
    type Database = Sqlite;

    fn begin(conn: &mut SqliteConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            conn.execute(&*begin_ansi_transaction_sql(depth)).await?;

            Ok(())
        })
    }

    fn commit(conn: &mut SqliteConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            conn.execute(&*commit_ansi_transaction_sql(depth)).await?;

            Ok(())
        })
    }

    fn rollback(conn: &mut SqliteConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            conn.execute(&*rollback_ansi_transaction_sql(depth)).await?;

            Ok(())
        })
    }

    fn start_rollback(conn: &mut SqliteConnection, depth: usize) {
        let query = rollback_ansi_transaction_sql(depth);
        let mut z_query = String::with_capacity(query.len() + 1);
        z_query.push_str(&query);
        z_query.push('\0');

        unsafe {
            // NOTE: this is a direct execution as a ROLLBACK is unlikely to block for any amount of time
            let status = sqlite3_exec(
                conn.handle.as_ptr(),
                z_query.as_ptr() as _,
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if status != SQLITE_OK {
                panic!(
                    "error occurred while dropping a transaction: {}",
                    SqliteError::new(conn.handle.as_ptr())
                );
            }
        }
    }
}
