use std::ptr::NonNull;

use libsqlite3_sys::{sqlite3, sqlite3_close, SQLITE_OK};

use crate::sqlite::SqliteError;
use std::sync::Arc;

/// Managed handle to the raw SQLite3 database handle.
/// The database handle will be closed when this is dropped and no `ConnectionHandleRef`s exist.
#[derive(Debug)]
pub(crate) struct ConnectionHandle(Arc<HandleInner>);

/// A wrapper around `ConnectionHandle` which only exists for a `StatementWorker` to own
/// which prevents the `sqlite3` handle from being finalized while it is running `sqlite3_step()`
/// or `sqlite3_reset()`.
///
/// Note that this does *not* actually give access to the database handle!
#[derive(Clone, Debug)]
pub(crate) struct ConnectionHandleRef(Arc<HandleInner>);

// Wrapper for `*mut sqlite3` which finalizes the handle on-drop.
#[derive(Debug)]
struct HandleInner(NonNull<sqlite3>);

// A SQLite3 handle is safe to send between threads, provided not more than
// one is accessing it at the same time. This is upheld as long as [SQLITE_CONFIG_MULTITHREAD] is
// enabled and [SQLITE_THREADSAFE] was enabled when sqlite was compiled. We refuse to work
// if these conditions are not upheld.

// <https://www.sqlite.org/c3ref/threadsafe.html>

// <https://www.sqlite.org/c3ref/c_config_covering_index_scan.html#sqliteconfigmultithread>

unsafe impl Send for ConnectionHandle {}

// SAFETY: `Arc<T>` normally only implements `Send` where `T: Sync` because it allows
// concurrent access.
//
// However, in this case we're only using `Arc` to prevent the database handle from being
// finalized while the worker still holds a statement handle; `ConnectionHandleRef` thus
// should *not* actually provide access to the database handle.
unsafe impl Send for ConnectionHandleRef {}

impl ConnectionHandle {
    #[inline]
    pub(super) unsafe fn new(ptr: *mut sqlite3) -> Self {
        Self(Arc::new(HandleInner(NonNull::new_unchecked(ptr))))
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut sqlite3 {
        self.0 .0.as_ptr()
    }

    #[inline]
    pub(crate) fn to_ref(&self) -> ConnectionHandleRef {
        ConnectionHandleRef(Arc::clone(&self.0))
    }
}

impl Drop for HandleInner {
    fn drop(&mut self) {
        unsafe {
            // https://sqlite.org/c3ref/close.html
            let status = sqlite3_close(self.0.as_ptr());
            if status != SQLITE_OK {
                // this should *only* happen due to an internal bug in SQLite where we left
                // SQLite handles open
                panic!("{}", SqliteError::new(self.0.as_ptr()));
            }
        }
    }
}
