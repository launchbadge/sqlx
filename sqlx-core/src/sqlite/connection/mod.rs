use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::future;
use hashbrown::HashMap;
use libsqlite3_sys::sqlite3;

use crate::common::StatementCache;
use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::sqlite::connection::establish::establish;
use crate::sqlite::statement::{SqliteStatement, StatementWorker};
use crate::sqlite::{Sqlite, SqliteConnectOptions};

mod establish;
mod executor;
mod handle;

pub(crate) use handle::ConnectionHandle;

/// A connection to a [Sqlite] database.
pub struct SqliteConnection {
    pub(crate) handle: ConnectionHandle,
    pub(crate) worker: StatementWorker,

    // cache of semi-persistent statements
    pub(crate) statements: StatementCache<SqliteStatement>,

    // most recent non-persistent statement
    pub(crate) statement: Option<SqliteStatement>,

    // working memory for the active row's column information
    scratch_row_column_names: Arc<HashMap<UStr, usize>>,
}

impl SqliteConnection {
    /// Returns the underlying sqlite3* connection handle
    pub fn as_raw_handle(&mut self) -> *mut sqlite3 {
        self.handle.as_ptr()
    }
}

impl Debug for SqliteConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteConnection").finish()
    }
}

impl Connection for SqliteConnection {
    type Database = Sqlite;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        // nothing explicit to do; connection will close in drop
        Box::pin(future::ok(()))
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite connections, PING does effectively nothing
        Box::pin(future::ok(()))
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

    #[doc(hidden)]
    fn get_ref(&self) -> &Self {
        self
    }

    #[doc(hidden)]
    fn get_mut(&mut self) -> &mut Self {
        self
    }
}

impl Connect for SqliteConnection {
    type Options = SqliteConnectOptions;

    #[inline]
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(async move {
            let conn = establish(options).await?;

            // TODO: Apply any connection options once we have them defined

            Ok(conn)
        })
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        // before the connection handle is dropped,
        // we must explicitly drop the statements as the drop-order in a struct is undefined
        self.statements.clear();
        self.statement.take();

        // we then explicitly close the worker
        self.worker.close();
    }
}
