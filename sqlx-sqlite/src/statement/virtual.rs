#![allow(clippy::rc_buffer)]

use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};
use std::sync::Arc;
use std::{cmp, i32};

use libsqlite3_sys::{
    sqlite3, sqlite3_prepare_v3, sqlite3_stmt, SQLITE_OK, SQLITE_PREPARE_PERSISTENT,
};

use sqlx_core::bytes::{Buf, Bytes};
use sqlx_core::error::Error;
use sqlx_core::ext::ustr::UStr;
use sqlx_core::{HashMap, SmallVec};

use crate::connection::ConnectionHandle;
use crate::statement::StatementHandle;
use crate::{SqliteColumn, SqliteError};

// A virtual statement consists of *zero* or more raw SQLite3 statements. We chop up a SQL statement
// on `;` to support multiple statements in one query.

#[derive(Debug)]
pub struct VirtualStatement {
    persistent: bool,

    /// the current index of the actual statement that is executing
    /// if `None`, no statement is executing and `prepare()` must be called;
    /// if `Some(self.handles.len())` and `self.tail.is_empty()`,
    /// there are no more statements to execute and `reset()` must be called
    index: Option<usize>,

    /// The full input SQL.
    sql: Arc<str>,

    /// The byte offset of the next statement to prepare in `sql`.
    tail_offset: usize,

    /// underlying sqlite handles for each inner statement
    /// a SQL query string in SQLite is broken up into N statements
    /// we use a [`SmallVec`] to optimize for the most likely case of a single statement
    pub(crate) handles: SmallVec<[StatementHandle; 1]>,

    // each set of columns
    pub(crate) columns: SmallVec<[Arc<Vec<SqliteColumn>>; 1]>,

    // each set of column names
    pub(crate) column_names: SmallVec<[Arc<HashMap<UStr, usize>>; 1]>,

    /// Offsets into `sql` for each statement.
    pub(crate) sql_offsets: SmallVec<[usize; 1]>,
}

pub struct PreparedStatement<'a> {
    pub(crate) handle: &'a mut StatementHandle,
    pub(crate) columns: &'a Arc<Vec<SqliteColumn>>,
    pub(crate) column_names: &'a Arc<HashMap<UStr, usize>>,
}

impl VirtualStatement {
    pub(crate) fn new(query: Arc<str>, persistent: bool) -> Result<Self, Error> {
        if query.len() > i32::max_value() as usize {
            return Err(err_protocol!(
                "query string must be smaller than {} bytes",
                i32::MAX
            ));
        }

        Ok(Self {
            persistent,
            sql: query,
            tail_offset: 0,
            handles: SmallVec::with_capacity(1),
            index: None,
            columns: SmallVec::with_capacity(1),
            column_names: SmallVec::with_capacity(1),
            sql_offsets: SmallVec::with_capacity(1),
        })
    }

    pub(crate) fn prepare_next(
        &mut self,
        conn: &mut ConnectionHandle,
    ) -> Result<Option<PreparedStatement<'_>>, Error> {
        // increment `self.index` up to `self.handles.len()`
        self.index = self
            .index
            .map(|idx| cmp::min(idx + 1, self.handles.len()))
            .or(Some(0));

        while self.handles.len() <= self.index.unwrap_or(0) {
            let sql_offset = self.tail_offset;

            let query = self.sql.get(sql_offset..).unwrap_or("");

            if query.is_empty() {
                return Ok(None);
            }

            let (consumed, maybe_statement) = try_prepare(
                conn.as_ptr(),
                query,
                self.persistent,
            ).map_err(|mut e| {
                // `sqlite3_offset()` returns the offset into the passed string,
                // but we want the offset into the original SQL string.
                e.add_offset(sql_offset);
                e.find_error_pos(&self.sql);
                e
            })?;

            self.tail_offset = self.tail_offset
                .checked_add(consumed)
                // Highly unlikely, but since we're dealing with `unsafe` here
                // it's best not to fool around.
                .ok_or_else(|| Error::Protocol(format!("overflow adding {n:?} bytes to tail_offset {tail_offset:?}")))?;

            if let Some(statement) = maybe_statement {
                let num = statement.column_count();

                let mut columns = Vec::with_capacity(num);
                let mut column_names = HashMap::with_capacity(num);

                for i in 0..num {
                    let name: UStr = statement.column_name(i).to_owned().into();
                    let type_info = statement
                        .column_decltype(i)
                        .unwrap_or_else(|| statement.column_type_info(i));

                    columns.push(SqliteColumn {
                        ordinal: i,
                        name: name.clone(),
                        type_info,
                    });

                    column_names.insert(name, i);
                }

                self.handles.push(statement);
                self.columns.push(Arc::new(columns));
                self.column_names.push(Arc::new(column_names));
                self.sql_offsets.push(sql_offset);
            }
        }

        Ok(self.current())
    }

    pub fn current(&mut self) -> Option<PreparedStatement<'_>> {
        self.index
            .filter(|&idx| idx < self.handles.len())
            .map(move |idx| PreparedStatement {
                handle: &mut self.handles[idx],
                columns: &self.columns[idx],
                column_names: &self.column_names[idx],
            })
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        self.index = None;

        for handle in self.handles.iter_mut() {
            handle.reset()?;
            handle.clear_bindings();
        }

        Ok(())
    }
}

/// Attempt to prepare one statement, returning the number of bytes consumed from `sql`,
/// and the statement handle if successful.
fn try_prepare(
    conn: *mut sqlite3,
    query: &str,
    persistent: bool,
) -> Result<(usize, Option<StatementHandle>), SqliteError> {
    let mut flags = 0;

    // For some reason, when building with the `sqlcipher` feature enabled
    // `SQLITE_PREPARE_PERSISTENT` ends up being `i32` instead of `u32`. Crazy, right?
    #[allow(trivial_casts, clippy::unnecessary_cast)]
    if persistent {
        // SQLITE_PREPARE_PERSISTENT
        //  The SQLITE_PREPARE_PERSISTENT flag is a hint to the query
        //  planner that the prepared statement will be retained for a long time
        //  and probably reused many times.
        flags |= SQLITE_PREPARE_PERSISTENT as u32;
    }

    let mut statement_handle: *mut sqlite3_stmt = null_mut();
    let mut tail_ptr: *const c_char = null();

    let query_ptr = query.as_ptr() as *const c_char;
    let query_len = query.len() as i32;

    // <https://www.sqlite.org/c3ref/prepare.html>
    let status = unsafe {
        sqlite3_prepare_v3(
            conn,
            query_ptr,
            query_len,
            flags,
            &mut statement_handle,
            &mut tail_ptr,
        )
    };

    if status != SQLITE_OK {
        // Note: `offset` and `error_pos` will be updated in `VirtualStatement::prepare_next()`.
        return Err(SqliteError::new(conn));
    }

    // tail should point to the first byte past the end of the first SQL
    // statement in zSql. these routines only compile the first statement,
    // so tail is left pointing to what remains un-compiled.

    let consumed = (tail_ptr as usize) - (query_ptr as usize);

    Ok((
        consumed,
        NonNull::new(statement_handle).map(StatementHandle::new),
    ))
}
