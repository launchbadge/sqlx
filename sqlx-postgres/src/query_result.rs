use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::str::Utf8Error;

use bytes::Bytes;
use bytestring::ByteString;
use memchr::memrchr;
use sqlx_core::{Error, QueryResult, Result};

use crate::PgClientError;

// TODO: add unit tests for command tag parsing

/// Represents the execution result of a command in Postgres.
///
/// Returned from [`execute()`][sqlx_core::Executor::execute].
///
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default)]
pub struct PgQueryResult {
    command: ByteString,
    rows_affected: u64,
}

impl PgQueryResult {
    pub(crate) fn parse(mut command: Bytes) -> Result<Self> {
        // look backwards for the first SPACE
        let offset = memrchr(b' ', &command);

        let rows = if let Some(offset) = offset {
            atoi::atoi(&command.split_off(offset).slice(1..)).unwrap_or(0)
        } else {
            0
        };

        let command: ByteString = command.try_into().map_err(PgClientError::NotUtf8)?;

        Ok(Self { command, rows_affected: rows })
    }

    /// Returns the command tag.
    ///
    /// This is usually a single word that identifies which SQL command
    /// was completed (e.g.,`INSERT`, `UPDATE`, or `MOVE`).
    ///
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Returns the number of rows inserted, deleted, updated, retrieved,
    /// changed, or copied by the SQL command.
    #[must_use]
    pub const fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Debug for PgQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgQueryResult")
            .field("command", &self.command())
            .field("rows_affected", &self.rows_affected())
            .finish()
    }
}

impl Extend<PgQueryResult> for PgQueryResult {
    fn extend<T: IntoIterator<Item = PgQueryResult>>(&mut self, iter: T) {
        for res in iter {
            self.rows_affected += res.rows_affected;
            self.command = res.command;
        }
    }
}

impl QueryResult for PgQueryResult {
    #[inline]
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}
