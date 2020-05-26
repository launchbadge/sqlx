use std::i32;
use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};
use std::sync::{atomic::AtomicPtr, Weak};

use bytes::{Buf, Bytes};
use libsqlite3_sys::{
    sqlite3, sqlite3_clear_bindings, sqlite3_finalize, sqlite3_prepare_v3, sqlite3_reset,
    sqlite3_stmt, SQLITE_OK, SQLITE_PREPARE_NO_VTAB, SQLITE_PREPARE_PERSISTENT,
};
use smallvec::SmallVec;

use crate::error::Error;
use crate::sqlite::connection::ConnectionHandle;
use crate::sqlite::{SqliteError, SqliteRow, SqliteValue};

mod handle;
mod worker;

pub(crate) use handle::StatementHandle;
pub(crate) use worker::StatementWorker;

// NOTE: Keep query in statement and slowly chop it up

#[derive(Debug)]
pub(crate) struct SqliteStatement {
    persistent: bool,
    index: usize,

    // tail of the most recently prepared SQL statement within this container
    tail: Bytes,

    // underlying sqlite handles for each inner statement
    // a SQL query string in SQLite is broken up into N statements
    // we use a [`SmallVec`] to optimize for the most likely case of a single statement
    pub(crate) handles: SmallVec<[StatementHandle; 1]>,

    // weak reference to the previous row from this connection
    // we use the notice of a successful upgrade of this reference as an indicator that the
    // row is still around, in which we then inflate the row such that we can let SQLite
    // clobber the memory allocation for the row
    pub(crate) last_row_values: SmallVec<[Option<Weak<AtomicPtr<SqliteValue>>>; 1]>,
}

fn prepare(
    conn: *mut sqlite3,
    query: &mut Bytes,
    persistent: bool,
) -> Result<Option<StatementHandle>, Error> {
    let mut flags = SQLITE_PREPARE_NO_VTAB;

    if persistent {
        // SQLITE_PREPARE_PERSISTENT
        //  The SQLITE_PREPARE_PERSISTENT flag is a hint to the query
        //  planner that the prepared statement will be retained for a long time
        //  and probably reused many times.
        flags |= SQLITE_PREPARE_PERSISTENT;
    }

    while !query.is_empty() {
        let mut statement_handle: *mut sqlite3_stmt = null_mut();
        let mut tail: *const c_char = null();

        let query_ptr = query.as_ptr() as *const c_char;
        let query_len = query.len() as i32;

        // <https://www.sqlite.org/c3ref/prepare.html>
        let status = unsafe {
            sqlite3_prepare_v3(
                conn,
                query_ptr,
                query_len,
                flags as u32,
                &mut statement_handle,
                &mut tail,
            )
        };

        if status != SQLITE_OK {
            return Err(SqliteError::new(conn).into());
        }

        // tail should point to the first byte past the end of the first SQL
        // statement in zSql. these routines only compile the first statement,
        // so tail is left pointing to what remains un-compiled.

        let n = (tail as i32) - (query_ptr as i32);
        query.advance(n as usize);

        if let Some(handle) = NonNull::new(statement_handle) {
            return Ok(Some(StatementHandle(handle)));
        }
    }

    Ok(None)
}

impl SqliteStatement {
    pub(crate) fn prepare(
        conn: &mut ConnectionHandle,
        mut query: &str,
        persistent: bool,
    ) -> Result<Self, Error> {
        query = query.trim();

        if query.len() > i32::MAX as usize {
            return Err(err_protocol!(
                "query string must be smaller than {} bytes",
                i32::MAX
            ));
        }

        let mut handles: SmallVec<[StatementHandle; 1]> = SmallVec::with_capacity(1);
        let mut query = Bytes::from(String::from(query));

        if let Some(handle) = prepare(conn.as_ptr(), &mut query, persistent)? {
            handles.push(handle);
        }

        Ok(Self {
            persistent,
            tail: query,
            handles,
            index: 0,
            last_row_values: SmallVec::from([None; 1]),
        })
    }

    // unsafe: caller must ensure that there is at least one handle
    unsafe fn connection(&self) -> *mut sqlite3 {
        self.handles[0].db_handle()
    }

    pub(crate) fn execute(
        &mut self,
    ) -> Result<Option<(&StatementHandle, &mut Option<Weak<AtomicPtr<SqliteValue>>>)>, Error> {
        while self.handles.len() == self.index {
            if self.tail.is_empty() {
                return Ok(None);
            }

            if let Some(handle) =
                unsafe { prepare(self.connection(), &mut self.tail, self.persistent)? }
            {
                self.handles.push(handle);
                self.last_row_values.push(None);
            }
        }

        let index = self.index;
        self.index += 1;

        Ok(Some((
            &self.handles[index],
            &mut self.last_row_values[index],
        )))
    }

    pub(crate) fn reset(&mut self) {
        self.index = 0;

        for (i, handle) in self.handles.iter().enumerate() {
            SqliteRow::inflate_if_needed(&handle, self.last_row_values[i].take());

            unsafe {
                // Reset A Prepared Statement Object
                // https://www.sqlite.org/c3ref/reset.html
                // https://www.sqlite.org/c3ref/clear_bindings.html
                sqlite3_reset(handle.0.as_ptr());
                sqlite3_clear_bindings(handle.0.as_ptr());
            }
        }
    }
}

impl Drop for SqliteStatement {
    fn drop(&mut self) {
        for (i, handle) in self.handles.drain(..).enumerate() {
            SqliteRow::inflate_if_needed(&handle, self.last_row_values[i].take());

            unsafe {
                // https://sqlite.org/c3ref/finalize.html
                let _ = sqlite3_finalize(handle.0.as_ptr());
            }
        }
    }
}
