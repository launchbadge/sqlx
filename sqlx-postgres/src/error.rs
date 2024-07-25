use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display, Formatter};

use atoi::atoi;

pub(crate) use sqlx_core::error::*;

use crate::message::{Notice, PgSeverity};

/// An error returned from the PostgreSQL database.
pub struct PgDatabaseError {
    pub(crate) notice: Notice,
    pub(crate) error_pos: Option<ErrorPosition>,
}

// Error message fields are documented:
// https://www.postgresql.org/docs/current/protocol-error-fields.html

impl PgDatabaseError {
    pub(crate) fn new(notice: Notice) -> Self {
        PgDatabaseError {
            notice,
            error_pos: None,
        }
    }

    pub(crate) fn find_error_pos(&mut self, query: &str) {
        let error_pos = self
            .pg_error_position()
            .and_then(|pos_raw| pos_raw.original())
            .and_then(|pos| ErrorPosition::find(query, PositionBasis::CharPos(pos)));

        self.error_pos = error_pos;
    }

    #[inline]
    pub fn severity(&self) -> PgSeverity {
        self.notice.severity()
    }

    /// The [SQLSTATE](https://www.postgresql.org/docs/current/errcodes-appendix.html) code for
    /// this error.
    #[inline]
    pub fn code(&self) -> &str {
        self.notice.code()
    }

    /// The primary human-readable error message. This should be accurate but
    /// terse (typically one line).
    #[inline]
    pub fn message(&self) -> &str {
        self.notice.message()
    }

    /// An optional secondary error message carrying more detail about the problem.
    /// Might run to multiple lines.
    #[inline]
    pub fn detail(&self) -> Option<&str> {
        self.notice.get(b'D')
    }

    /// An optional suggestion what to do about the problem. This is intended to differ from
    /// `detail` in that it offers advice (potentially inappropriate) rather than hard facts.
    /// Might run to multiple lines.
    #[inline]
    pub fn hint(&self) -> Option<&str> {
        self.notice.get(b'H')
    }

    /// Indicates an error cursor position as a 1-based index into the original query string; or,
    /// a position into an internally generated query.
    #[inline]
    pub fn pg_error_position(&self) -> Option<PgErrorPosition<'_>> {
        self.notice
            .get_raw(b'P')
            .and_then(atoi)
            .map(PgErrorPosition::Original)
            .or_else(|| {
                let position = self.notice.get_raw(b'p').and_then(atoi)?;
                let query = self.notice.get(b'q')?;

                Some(PgErrorPosition::Internal { position, query })
            })
    }

    /// An indication of the context in which the error occurred. Presently this includes a call
    /// stack traceback of active procedural language functions and internally-generated queries.
    /// The trace is one entry per line, most recent first.
    pub fn r#where(&self) -> Option<&str> {
        self.notice.get(b'W')
    }

    /// If this error is with a specific database object, the
    /// name of the schema containing that object, if any.
    pub fn schema(&self) -> Option<&str> {
        self.notice.get(b's')
    }

    /// If this error is with a specific table, the name of the table.
    pub fn table(&self) -> Option<&str> {
        self.notice.get(b't')
    }

    /// If the error is with a specific table column, the name of the column.
    pub fn column(&self) -> Option<&str> {
        self.notice.get(b'c')
    }

    /// If the error is with a specific data type, the name of the data type.
    pub fn data_type(&self) -> Option<&str> {
        self.notice.get(b'd')
    }

    /// If the error is with a specific constraint, the name of the constraint.
    /// For this purpose, indexes are constraints, even if they weren't created
    /// with constraint syntax.
    pub fn constraint(&self) -> Option<&str> {
        self.notice.get(b'n')
    }

    /// The file name of the server source-code location where this error was reported.
    pub fn file(&self) -> Option<&str> {
        self.notice.get(b'F')
    }

    /// The line number of the server source-code location where this error was reported.
    pub fn line(&self) -> Option<usize> {
        self.notice.get_raw(b'L').and_then(atoi)
    }

    /// The name of the server source-code routine reporting this error.
    pub fn routine(&self) -> Option<&str> {
        self.notice.get(b'R')
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum PgErrorPosition<'a> {
    /// A 1-based position (in characters) into the original query.
    Original(usize),

    /// A position into the internally-generated query.
    Internal {
        /// The 1-based position, in characters.
        position: usize,

        /// The text of a failed internally-generated command. This could be, for example,
        /// the SQL query issued by a PL/pgSQL function.
        query: &'a str,
    },
}

impl Debug for PgDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgDatabaseError")
            .field("position", &self.error_pos)
            .field("severity", &self.severity())
            .field("code", &self.code())
            .field("message", &self.message())
            .field("detail", &self.detail())
            .field("hint", &self.hint())
            .field("char_position", &self.pg_error_position())
            .field("where", &self.r#where())
            .field("schema", &self.schema())
            .field("table", &self.table())
            .field("column", &self.column())
            .field("data_type", &self.data_type())
            .field("constraint", &self.constraint())
            .field("file", &self.file())
            .field("line", &self.line())
            .field("routine", &self.routine())
            .finish()
    }
}

impl Display for PgDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(code {}", self.code())?;
        if let Some(error_pos) = self.error_pos {
            write!(f, ", line {}, column {}", error_pos.line, error_pos.column)?;
        }

        write!(f, ") {}", self.message())
    }
}

impl StdError for PgDatabaseError {}

impl DatabaseError for PgDatabaseError {
    fn message(&self) -> &str {
        self.message()
    }

    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(self.code()))
    }

    fn position(&self) -> Option<ErrorPosition> {
        self.error_pos
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> BoxDynError {
        self
    }

    fn is_transient_in_connect_phase(&self) -> bool {
        // https://www.postgresql.org/docs/current/errcodes-appendix.html
        [
            // too_many_connections
            // This may be returned if we just un-gracefully closed a connection,
            // give the database a chance to notice it and clean it up.
            "53300",
            // cannot_connect_now
            // Returned if the database is still starting up.
            "57P03",
        ]
        .contains(&self.code())
    }

    fn constraint(&self) -> Option<&str> {
        self.constraint()
    }

    fn table(&self) -> Option<&str> {
        self.table()
    }

    fn kind(&self) -> ErrorKind {
        match self.code() {
            error_codes::UNIQUE_VIOLATION => ErrorKind::UniqueViolation,
            error_codes::FOREIGN_KEY_VIOLATION => ErrorKind::ForeignKeyViolation,
            error_codes::NOT_NULL_VIOLATION => ErrorKind::NotNullViolation,
            error_codes::CHECK_VIOLATION => ErrorKind::CheckViolation,
            _ => ErrorKind::Other,
        }
    }
}

impl PgErrorPosition<'_> {
    fn original(&self) -> Option<usize> {
        match *self {
            Self::Original(original) => Some(original),
            _ => None,
        }
    }
}

pub(crate) trait PgResultExt {
    fn pg_find_error_pos(self, query: &str) -> Self;
}

impl<T> PgResultExt for Result<T, Error> {
    fn pg_find_error_pos(self, query: &str) -> Self {
        self.map_err(|e| {
            match e {
                Error::Database(e) => {
                    Error::Database(
                        // Don't panic in case this gets called in the wrong context;
                        // it'd be a bug, for sure, but far from a fatal one.
                        // The trait method has a distinct name to call this out if it happens.
                        e.try_downcast::<PgDatabaseError>().map_or_else(
                            |e| e,
                            |mut e| {
                                e.find_error_pos(query);
                                e
                            },
                        ),
                    )
                }
                other => other,
            }
        })
    }
}

/// For reference: <https://www.postgresql.org/docs/current/errcodes-appendix.html>
pub(crate) mod error_codes {
    /// Caused when a unique or primary key is violated.
    pub const UNIQUE_VIOLATION: &str = "23505";
    /// Caused when a foreign key is violated.
    pub const FOREIGN_KEY_VIOLATION: &str = "23503";
    /// Caused when a column marked as NOT NULL received a null value.
    pub const NOT_NULL_VIOLATION: &str = "23502";
    /// Caused when a check constraint is violated.
    pub const CHECK_VIOLATION: &str = "23514";
}
