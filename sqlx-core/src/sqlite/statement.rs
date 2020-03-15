use core::ptr::{null, null_mut, NonNull};

use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_int;

use libsqlite3_sys::{
    sqlite3_bind_parameter_count, sqlite3_column_count, sqlite3_column_decltype,
    sqlite3_column_name, sqlite3_finalize, sqlite3_prepare_v3, sqlite3_reset, sqlite3_step,
    sqlite3_stmt, SQLITE_DONE, SQLITE_OK, SQLITE_PREPARE_NO_VTAB, SQLITE_PREPARE_PERSISTENT,
    SQLITE_ROW,
};

use crate::sqlite::worker::Worker;
use crate::sqlite::SqliteError;
use crate::sqlite::{SqliteArguments, SqliteConnection};

pub(crate) enum Step {
    Done,
    Row,
}

#[derive(Clone, Copy)]
pub(super) struct SqliteStatementHandle(NonNull<sqlite3_stmt>);

pub(super) struct SqliteStatement {
    worker: Worker,
    pub(super) tail: usize,
    pub(super) handle: SqliteStatementHandle,
    pub(super) columns: HashMap<String, usize>,
}

// SAFE: See notes for the Send impl on [SqliteConnection].

#[allow(unsafe_code)]
unsafe impl Send for SqliteStatementHandle {}

#[allow(unsafe_code)]
unsafe impl Sync for SqliteStatementHandle {}

impl SqliteStatement {
    pub(super) fn handle(&self) -> *mut sqlite3_stmt {
        self.handle.0.as_ptr()
    }

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
        #[allow(unsafe_code)]
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

        // If pzTail is not NULL then *pzTail is made to point to the first byte
        // past the end of the first SQL statement in zSql.
        let tail = (tail as usize) - (query_ptr as usize);
        *query = &query[tail..].trim();

        if status != SQLITE_OK {
            return Err(SqliteError::new(status).into());
        }

        let mut self_ = Self {
            worker: conn.worker.clone(),
            handle: SqliteStatementHandle(NonNull::new(statement_handle).unwrap()),
            columns: HashMap::new(),
            tail,
        };

        // Prepare a column hash map for use in pulling values from a column by name
        let count = self_.num_columns();
        self_.columns.reserve(count);

        for i in 0..count {
            self_.columns.insert(self_.column_name(i).to_owned(), i);
        }

        Ok(self_)
    }

    pub(super) fn num_columns(&self) -> usize {
        // https://sqlite.org/c3ref/column_count.html
        #[allow(unsafe_code)]
        let count = unsafe { sqlite3_column_count(self.handle()) };
        count as usize
    }

    pub(super) fn column_name(&self, index: usize) -> &str {
        // https://sqlite.org/c3ref/column_name.html
        #[allow(unsafe_code)]
        let name = unsafe {
            let ptr = sqlite3_column_name(self.handle(), index as c_int);
            debug_assert!(!ptr.is_null());

            CStr::from_ptr(ptr)
        };

        name.to_str().unwrap()
    }

    pub(super) fn column_decltype(&self, index: usize) -> Option<&str> {
        // https://sqlite.org/c3ref/column_name.html
        #[allow(unsafe_code)]
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

    pub(super) fn params(&self) -> usize {
        // https://www.hwaci.com/sw/sqlite/c3ref/bind_parameter_count.html
        #[allow(unsafe_code)]
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

        // the status value of reset is ignored because it merely propagates
        // the status of the most recently invoked step function

        #[allow(unsafe_code)]
        let _ = unsafe { sqlite3_reset(self.handle()) };
    }

    pub(super) async fn step(&mut self) -> crate::Result<Step> {
        // https://sqlite.org/c3ref/step.html

        let status = self
            .worker
            .run({
                let handle = self.handle;
                move || {
                    #[allow(unsafe_code)]
                    unsafe {
                        sqlite3_step(handle.0.as_ptr())
                    }
                }
            })
            .await;

        match status {
            SQLITE_DONE => Ok(Step::Done),
            SQLITE_ROW => Ok(Step::Row),

            status => {
                return Err(SqliteError::new(status).into());
            }
        }
    }
}

impl Drop for SqliteStatement {
    fn drop(&mut self) {
        // https://sqlite.org/c3ref/finalize.html
        #[allow(unsafe_code)]
        unsafe {
            let _ = sqlite3_finalize(self.handle());
        }
    }
}
