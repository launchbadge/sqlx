use crate::common::StatementCache;
use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::sqlite::statement::{StatementWorker, VirtualStatement};
use crate::sqlite::{Sqlite, SqliteConnectOptions};
use crate::transaction::Transaction;
use futures_core::future::BoxFuture;
use futures_util::future;
use libsqlite3_sys::sqlite3;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};

mod collation;
mod describe;
pub(crate) mod establish;
mod executor;
mod explain;
mod handle;

pub(crate) use handle::{ConnectionHandle, ConnectionHandleRef};

/// A connection to a [Sqlite] database.
pub struct SqliteConnection {
    pub(crate) handle: ConnectionHandle,
    pub(crate) worker: StatementWorker,

    // transaction status
    pub(crate) transaction_depth: usize,

    // cache of semi-persistent statements
    pub(crate) statements: StatementCache<VirtualStatement>,

    // most recent non-persistent statement
    pub(crate) statement: Option<VirtualStatement>,

    log_settings: LogSettings,
}

impl SqliteConnection {
    /// Returns the underlying sqlite3* connection handle
    pub fn as_raw_handle(&mut self) -> *mut sqlite3 {
        self.handle.as_ptr()
    }

    pub fn create_collation(
        &mut self,
        name: &str,
        compare: impl Fn(&str, &str) -> Ordering + Send + Sync + 'static,
    ) -> Result<(), Error> {
        collation::create_collation(&self.handle, name, compare)
    }
}

impl Debug for SqliteConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteConnection").finish()
    }
}

impl Connection for SqliteConnection {
    type Database = Sqlite;

    type Options = SqliteConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            let shutdown = self.worker.shutdown();
            // Drop the statement worker and any outstanding statements, which should
            // cover all references to the connection handle outside of the worker thread
            drop(self);
            // Ensure the worker thread has terminated
            shutdown.await
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite connections, PING does effectively nothing
        Box::pin(future::ok(()))
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    fn cached_statements_size(&self) -> usize {
        self.statements.len()
    }

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.statements.clear();
            Ok(())
        })
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite, FLUSH does effectively nothing
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        false
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        // explicitly drop statements before the connection handle is dropped
        self.statements.clear();
        self.statement.take();
    }
}
