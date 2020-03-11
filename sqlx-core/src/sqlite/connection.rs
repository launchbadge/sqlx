use core::ptr::{NonNull, null, null_mut};

use std::convert::TryInto;
use std::ffi::CString;
use std::fmt::{self, Debug};

use futures_core::future::BoxFuture;
use libsqlite3_sys::{
    sqlite3, sqlite3_open_v2, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_NOMUTEX,
    SQLITE_OPEN_READWRITE, SQLITE_OPEN_SHAREDCACHE,
};

use crate::runtime::spawn_blocking;
use crate::connection::{Connect, Connection};
use crate::url::Url;
use futures_util::future;
use crate::sqlite::SqliteError;

#[derive(Debug)]
pub struct SqliteConnection {
    pub(super) handle: NonNull<sqlite3>,
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
    let url = url.as_str().trim_start_matches("sqlite://");

    // By default, we connect to an in-memory database.
    // TODO: Handle the error when there are internal NULs in the database URL
    let filename = CString::new(url).unwrap();
    let mut handle = null_mut();

    // [SQLITE_OPEN_NOMUTEX] will instruct [sqlite3_open_v2] to return an error if it
    // cannot satisfy our wish for a thread-safe, lock-free connection object
    let flags = SQLITE_OPEN_READWRITE
        | SQLITE_OPEN_CREATE
        | SQLITE_OPEN_NOMUTEX
        | SQLITE_OPEN_SHAREDCACHE;

    // <https://www.sqlite.org/c3ref/open.html>
    #[allow(unsafe_code)]
    let status = unsafe {
        sqlite3_open_v2(filename.as_ptr(), &mut handle, flags, null())
    };

    if status != SQLITE_OK {
        return Err(SqliteError::new(status).into());
    }

    Ok(SqliteConnection {
        handle: NonNull::new(handle).unwrap(),
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
        // Box::pin(terminate(self.stream))
        todo!()
    }

    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        // For SQLite connections, PING does effectively nothing
        Box::pin(future::ok(()))
    }
}
