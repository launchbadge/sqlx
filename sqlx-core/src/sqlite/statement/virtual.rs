#![allow(clippy::rc_buffer)]

use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::sqlite::connection::ConnectionHandle;
use crate::sqlite::statement::StatementHandle;
use crate::sqlite::{SqliteColumn, SqliteError};
use crate::HashMap;
use bytes::{Buf, Bytes};
use libsqlite3_sys::{
    sqlite3, sqlite3_prepare_v3, sqlite3_stmt, SQLITE_OK, SQLITE_PREPARE_PERSISTENT,
};
use smallvec::SmallVec;
use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};
use std::sync::Arc;
use std::{cmp, i32};

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

    /// tail of the most recently prepared SQL statement within this container
    tail: Bytes,

    /// underlying sqlite handles for each inner statement
    /// a SQL query string in SQLite is broken up into N statements
    /// we use a [`SmallVec`] to optimize for the most likely case of a single statement
    pub(crate) handles: SmallVec<[StatementHandle; 1]>,

    // each set of columns
    pub(crate) columns: SmallVec<[Arc<Vec<SqliteColumn>>; 1]>,

    // each set of column names
    pub(crate) column_names: SmallVec<[Arc<HashMap<UStr, usize>>; 1]>,
}

pub struct PreparedStatement<'a> {
    pub(crate) handle: &'a mut StatementHandle,
    pub(crate) columns: &'a Arc<Vec<SqliteColumn>>,
    pub(crate) column_names: &'a Arc<HashMap<UStr, usize>>,
}

impl VirtualStatement {
    pub(crate) fn new(mut query: &str, persistent: bool) -> Result<Self, Error> {
        query = query.trim();

        if query.len() > i32::max_value() as usize {
            return Err(err_protocol!(
                "query string must be smaller than {} bytes",
                i32::MAX
            ));
        }

        Ok(Self {
            persistent,
            tail: Bytes::from(String::from(query)),
            handles: SmallVec::with_capacity(1),
            index: None,
            columns: SmallVec::with_capacity(1),
            column_names: SmallVec::with_capacity(1),
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
            if self.tail.is_empty() {
                return Ok(None);
            }

            if let Some(statement) = prepare(conn.as_ptr(), &mut self.tail, self.persistent)? {
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

fn prepare(
    conn: *mut sqlite3,
    query: &mut Bytes,
    persistent: bool,
) -> Result<Option<StatementHandle>, Error> {
    let mut flags = 0;

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

        let n = (tail as usize) - (query_ptr as usize);
        query.advance(n);

        if let Some(handle) = NonNull::new(statement_handle) {
            return Ok(Some(StatementHandle::new(handle)));
        }
    }

    Ok(None)
}
