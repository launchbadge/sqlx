use core::ptr::{null, null_mut, NonNull};

use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CString;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::future;
use libsqlite3_sys::{
    sqlite3, sqlite3_close, sqlite3_open_v2, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_READWRITE, SQLITE_OPEN_SHAREDCACHE,
};

use crate::connection::{Connect, Connection};
use crate::runtime::spawn_blocking;
use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::SqliteError;
use crate::url::Url;

pub struct SqliteConnection {
    pub(super) handle: NonNull<sqlite3>,
    pub(super) statements: Vec<SqliteStatement>,
    pub(super) statement_by_query: HashMap<String, usize>,
    pub(super) columns_by_query: HashMap<String, Arc<HashMap<String, usize>>>,
}

// SAFE: A sqlite3 handle is safe to access from multiple threads provided
//       that only one thread access it at a time. Or in other words,
//       the same guarantees that [Sync] requires. This is upheld as long
//       [SQLITE_CONFIG_MULTITHREAD] is enabled and [SQLITE_THREADSAFE] was
//       enabled when sqlite was compiled. We refuse to work if these conditions are
//       not upheld, see [SqliteConnection::establish].
//
// <https://www.sqlite.org/c3ref/threadsafe.html>
// <https://www.sqlite.org/c3ref/c_config_covering_index_scan.html#sqliteconfigmultithread>

#[allow(unsafe_code)]
unsafe impl Send for SqliteConnection {}

#[allow(unsafe_code)]
unsafe impl Sync for SqliteConnection {}

fn establish(url: crate::Result<Url>) -> crate::Result<SqliteConnection> {
    let url = url?;
    let url = url
        .as_str()
        .trim_start_matches("sqlite:")
        .trim_start_matches("//");

    // By default, we connect to an in-memory database.
    // TODO: Handle the error when there are internal NULs in the database URL
    let filename = CString::new(url).unwrap();
    let mut handle = null_mut();

    // [SQLITE_OPEN_NOMUTEX] will instruct [sqlite3_open_v2] to return an error if it
    // cannot satisfy our wish for a thread-safe, lock-free connection object
    let flags =
        SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX | SQLITE_OPEN_SHAREDCACHE;

    // <https://www.sqlite.org/c3ref/open.html>
    #[allow(unsafe_code)]
    let status = unsafe { sqlite3_open_v2(filename.as_ptr(), &mut handle, flags, null()) };

    if status != SQLITE_OK {
        return Err(SqliteError::new(status).into());
    }

    Ok(SqliteConnection {
        handle: NonNull::new(handle).unwrap(),
        statements: Vec::with_capacity(10),
        statement_by_query: HashMap::with_capacity(10),
        columns_by_query: HashMap::new(),
    })
}

impl Connect for SqliteConnection {
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<SqliteConnection>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        let url = url.try_into();
        Box::pin(spawn_blocking(move || establish(url)))
    }
}

impl Connection for SqliteConnection {
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        // All necessary behavior is handled on drop
        Box::pin(future::ok(()))
    }

    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        // For SQLite connections, PING does effectively nothing
        Box::pin(future::ok(()))
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        // Drop all statements first
        self.statements.clear();

        // Next close the statement
        // https://sqlite.org/c3ref/close.html
        #[allow(unsafe_code)]
        unsafe {
            let _ = sqlite3_close(self.handle.as_ptr());
        }
    }
}
