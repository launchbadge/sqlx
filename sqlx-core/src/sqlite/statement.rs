#![allow(unsafe_code)]

use core::ptr::{null, null_mut, NonNull};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_int;
use std::ptr;

use libsqlite3_sys::{
    sqlite3_bind_parameter_count, sqlite3_clear_bindings, sqlite3_column_count,
    sqlite3_column_database_name, sqlite3_column_decltype, sqlite3_column_name,
    sqlite3_column_origin_name, sqlite3_column_table_name, sqlite3_data_count, sqlite3_finalize,
    sqlite3_prepare_v3, sqlite3_reset, sqlite3_step, sqlite3_stmt, sqlite3_table_column_metadata,
    SQLITE_DONE, SQLITE_OK, SQLITE_PREPARE_NO_VTAB, SQLITE_PREPARE_PERSISTENT, SQLITE_ROW,
};

use crate::sqlite::connection::SqliteConnectionHandle;
use crate::sqlite::worker::Worker;
use crate::sqlite::SqliteError;
use crate::sqlite::{SqliteArguments, SqliteConnection};

/// Return values from [SqliteStatement::step].
pub(super) enum Step {
    /// The statement has finished executing successfully.
    Done,

    /// Another row of output is available.
    Row,
}

/// Thin wrapper around [sqlite3_stmt] to impl `Send`.
#[derive(Clone, Copy)]
pub(super) struct SqliteStatementHandle(NonNull<sqlite3_stmt>);

/// Represents a _single_ SQL statement that has been compiled into binary
/// form and is ready to be evaluated.
///
/// The statement is finalized ( `sqlite3_finalize` ) on drop.
pub(super) struct Statement {
    handle: SqliteStatementHandle,
    pub(super) connection: SqliteConnectionHandle,
    pub(super) worker: Worker,
    pub(super) tail: usize,
    pub(super) columns: HashMap<String, usize>,
}

// SQLite3 statement objects are safe to send between threads, but *not* safe
// for general-purpose concurrent access between threads. See more notes
// on [SqliteConnectionHandle].
unsafe impl Send for SqliteStatementHandle {}

impl Statement {
    pub(super) fn new(
        conn: &mut SqliteConnection,
        query: &mut &str,
        persistent: bool,
    ) -> crate::Result<Self> {
        // TODO: Error on queries that are too large
        let query_ptr = query.as_bytes().as_ptr() as *const i8;
        let query_len = query.len() as i32;
        let mut statement_handle: *mut sqlite3_stmt = null_mut();
        let mut flags = SQLITE_PREPARE_NO_VTAB;
        let mut tail: *const i8 = null();

        if persistent {
            // SQLITE_PREPARE_PERSISTENT
            //  The SQLITE_PREPARE_PERSISTENT flag is a hint to the query
            //  planner that the prepared statement will be retained for a long time
            //  and probably reused many times.
            flags |= SQLITE_PREPARE_PERSISTENT;
        }

        // <https://www.sqlite.org/c3ref/prepare.html>
        let status = unsafe {
            sqlite3_prepare_v3(
                conn.handle(),
                query_ptr,
                query_len,
                flags as u32,
                &mut statement_handle,
                &mut tail,
            )
        };

        if status != SQLITE_OK {
            return Err(SqliteError::from_connection(conn.handle()).into());
        }

        // If pzTail is not NULL then *pzTail is made to point to the first byte
        // past the end of the first SQL statement in zSql.
        let tail = (tail as usize) - (query_ptr as usize);
        *query = &query[tail..].trim();

        let mut self_ = Self {
            worker: conn.worker.clone(),
            connection: conn.handle,
            handle: SqliteStatementHandle(NonNull::new(statement_handle).unwrap()),
            columns: HashMap::new(),
            tail,
        };

        // Prepare a column hash map for use in pulling values from a column by name
        let count = self_.column_count();
        self_.columns.reserve(count);

        for i in 0..count {
            let name = self_.column_name(i).to_owned();
            self_.columns.insert(name, i);
        }

        Ok(self_)
    }

    /// Returns a pointer to the raw C pointer backing this statement.
    #[inline]
    pub(super) unsafe fn handle(&self) -> *mut sqlite3_stmt {
        self.handle.0.as_ptr()
    }

    pub(super) fn data_count(&mut self) -> usize {
        // https://sqlite.org/c3ref/data_count.html

        // The sqlite3_data_count(P) interface returns the number of columns
        // in the current row of the result set.

        // The value is correct only if there was a recent call to
        // sqlite3_step that returned SQLITE_ROW.

        let count: c_int = unsafe { sqlite3_data_count(self.handle()) };
        count as usize
    }

    pub(super) fn column_count(&mut self) -> usize {
        // https://sqlite.org/c3ref/column_count.html
        let count = unsafe { sqlite3_column_count(self.handle()) };
        count as usize
    }

    pub(super) fn column_name(&mut self, index: usize) -> &str {
        // https://sqlite.org/c3ref/column_name.html
        let name = unsafe {
            let ptr = sqlite3_column_name(self.handle(), index as c_int);
            debug_assert!(!ptr.is_null());

            CStr::from_ptr(ptr)
        };

        name.to_str().unwrap()
    }

    pub(super) fn column_decltype(&mut self, index: usize) -> Option<&str> {
        let name = unsafe {
            let ptr = sqlite3_column_decltype(self.handle(), index as c_int);

            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr))
            }
        };

        name.map(|s| s.to_str().unwrap())
    }

    pub(super) fn column_not_null(&mut self, index: usize) -> crate::Result<Option<bool>> {
        unsafe {
            // https://sqlite.org/c3ref/column_database_name.html
            //
            // ### Note
            // The returned string is valid until the prepared statement is destroyed using
            // sqlite3_finalize() or until the statement is automatically reprepared by the
            // first call to sqlite3_step() for a particular run or until the same information
            // is requested again in a different encoding.
            let db_name = sqlite3_column_database_name(self.handle(), index as c_int);
            let table_name = sqlite3_column_table_name(self.handle(), index as c_int);
            let origin_name = sqlite3_column_origin_name(self.handle(), index as c_int);

            if db_name.is_null() || table_name.is_null() || origin_name.is_null() {
                return Ok(None);
            }

            let mut not_null: c_int = 0;

            // https://sqlite.org/c3ref/table_column_metadata.html
            let status = sqlite3_table_column_metadata(
                self.connection.0.as_ptr(),
                db_name,
                table_name,
                origin_name,
                // function docs state to provide NULL for return values you don't care about
                ptr::null_mut(),
                ptr::null_mut(),
                &mut not_null,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if status != SQLITE_OK {
                // implementation note: the docs for sqlite3_table_column_metadata() specify
                // that an error can be returned if the column came from a view; however,
                // experimentally we found that the above functions give us the true origin
                // for columns in views that came from real tables and so we should never hit this
                // error; for view columns that are expressions we are given NULL for their origins
                // so we don't need special handling for that case either.
                //
                // this is confirmed in the `tests/sqlite-macros.rs` integration test
                return Err(SqliteError::from_connection(self.connection.0.as_ptr()).into());
            }

            Ok(Some(not_null != 0))
        }
    }

    pub(super) fn params(&mut self) -> usize {
        // https://www.hwaci.com/sw/sqlite/c3ref/bind_parameter_count.html
        let num = unsafe { sqlite3_bind_parameter_count(self.handle()) };
        num as usize
    }

    pub(super) fn bind(&mut self, arguments: &mut SqliteArguments) -> crate::Result<()> {
        for index in 0..self.params() {
            if let Some(value) = arguments.next() {
                value.bind(self, index + 1)?;
            } else {
                break;
            }
        }

        Ok(())
    }

    pub(super) fn reset(&mut self) {
        // https://sqlite.org/c3ref/reset.html
        // https://sqlite.org/c3ref/clear_bindings.html

        // the status value of reset is ignored because it merely propagates
        // the status of the most recently invoked step function

        let _ = unsafe { sqlite3_reset(self.handle()) };

        let _ = unsafe { sqlite3_clear_bindings(self.handle()) };
    }

    pub(super) async fn step(&mut self) -> crate::Result<Step> {
        // https://sqlite.org/c3ref/step.html

        let handle = self.handle;

        let status = unsafe {
            self.worker
                .run(move || sqlite3_step(handle.0.as_ptr()))
                .await
        };

        match status {
            SQLITE_DONE => Ok(Step::Done),

            SQLITE_ROW => Ok(Step::Row),

            _ => {
                return Err(SqliteError::from_connection(self.connection.0.as_ptr()).into());
            }
        }
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        // https://sqlite.org/c3ref/finalize.html
        unsafe {
            let _ = sqlite3_finalize(self.handle());
        }
    }
}
