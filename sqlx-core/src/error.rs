//! Types for working with errors produced by SQLx.

use std::any::type_name;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::Display;
use std::io;

use crate::database::Database;

use crate::type_info::TypeInfo;
use crate::types::Type;

/// A specialized `Result` type for SQLx.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

// Convenience type alias for usage within SQLx.
// Do not make this type public.
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// An unexpected `NULL` was encountered during decoding.
///
/// Returned from [`Row::get`](crate::row::Row::get) if the value from the database is `NULL`,
/// and you are not decoding into an `Option`.
#[derive(thiserror::Error, Debug)]
#[error("unexpected null; try decoding as an `Option`")]
pub struct UnexpectedNullError;

/// Represents all the ways a method can fail within SQLx.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Error occurred while parsing a connection string.
    #[error("error with configuration: {0}")]
    Configuration(#[source] BoxDynError),

    /// Error returned from the database.
    #[error("error returned from database: {0}")]
    Database(#[source] Box<dyn DatabaseError>),

    /// Error communicating with the database backend.
    #[error("error communicating with database: {0}")]
    Io(#[from] io::Error),

    /// Error occurred while attempting to establish a TLS connection.
    #[error("error occurred while attempting to establish a TLS connection: {0}")]
    Tls(#[source] BoxDynError),

    /// Unexpected or invalid data encountered while communicating with the database.
    ///
    /// This should indicate there is a programming error in a SQLx driver or there
    /// is something corrupted with the connection to the database itself.
    #[error("encountered unexpected or invalid data: {0}")]
    Protocol(String),

    /// No rows returned by a query that expected to return at least one row.
    #[error("no rows returned by a query that expected to return at least one row")]
    RowNotFound,

    /// Type in query doesn't exist. Likely due to typo or missing user type.
    #[error("type named {type_name} not found")]
    TypeNotFound { type_name: String },

    /// Column index was out of bounds.
    #[error("column index out of bounds: the len is {len}, but the index is {index}")]
    ColumnIndexOutOfBounds { index: usize, len: usize },

    /// No column found for the given name.
    #[error("no column found for name: {0}")]
    ColumnNotFound(String),

    /// Error occurred while decoding a value from a specific column.
    #[error("error occurred while decoding column {index}: {source}")]
    ColumnDecode {
        index: String,

        #[source]
        source: BoxDynError,
    },

    /// Error occured while encoding a value.
    #[error("error occured while encoding a value: {0}")]
    Encode(#[source] BoxDynError),

    /// Error occurred while decoding a value.
    #[error("error occurred while decoding: {0}")]
    Decode(#[source] BoxDynError),

    /// Error occurred within the `Any` driver mapping to/from the native driver.
    #[error("error in Any driver mapping: {0}")]
    AnyDriverError(#[source] BoxDynError),

    /// A [`Pool::acquire`] timed out due to connections not becoming available or
    /// because another task encountered too many errors while trying to open a new connection.
    ///
    /// [`Pool::acquire`]: crate::pool::Pool::acquire
    #[error("pool timed out while waiting for an open connection")]
    PoolTimedOut,

    /// [`Pool::close`] was called while we were waiting in [`Pool::acquire`].
    ///
    /// [`Pool::acquire`]: crate::pool::Pool::acquire
    /// [`Pool::close`]: crate::pool::Pool::close
    #[error("attempted to acquire a connection on a closed pool")]
    PoolClosed,

    /// A background worker has crashed.
    #[error("attempted to communicate with a crashed background worker")]
    WorkerCrashed,

    #[cfg(feature = "migrate")]
    #[error("{0}")]
    Migrate(#[source] Box<crate::migrate::MigrateError>),
}

impl StdError for Box<dyn DatabaseError> {}

impl Error {
    pub fn into_database_error(self) -> Option<Box<dyn DatabaseError + 'static>> {
        match self {
            Error::Database(err) => Some(err),
            _ => None,
        }
    }

    pub fn as_database_error(&self) -> Option<&(dyn DatabaseError + 'static)> {
        match self {
            Error::Database(err) => Some(&**err),
            _ => None,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn protocol(err: impl Display) -> Self {
        Error::Protocol(err.to_string())
    }

    #[doc(hidden)]
    #[inline]
    pub fn config(err: impl StdError + Send + Sync + 'static) -> Self {
        Error::Configuration(err.into())
    }

    pub(crate) fn tls(err: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        Error::Tls(err.into())
    }

    #[doc(hidden)]
    #[inline]
    pub fn decode(err: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        Error::Decode(err.into())
    }
}

pub fn mismatched_types<DB: Database, T: Type<DB>>(ty: &DB::TypeInfo) -> BoxDynError {
    // TODO: `#name` only produces `TINYINT` but perhaps we want to show `TINYINT(1)`
    format!(
        "mismatched types; Rust type `{}` (as SQL type `{}`) is not compatible with SQL type `{}`",
        type_name::<T>(),
        T::type_info().name(),
        ty.name()
    )
    .into()
}

/// The error kind.
///
/// This enum is to be used to identify frequent errors that can be handled by the program.
/// Although it currently only supports constraint violations, the type may grow in the future.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Unique/primary key constraint violation.
    UniqueViolation,
    /// Foreign key constraint violation.
    ForeignKeyViolation,
    /// Not-null constraint violation.
    NotNullViolation,
    /// Check constraint violation.
    CheckViolation,
    /// An unmapped error.
    Other,
}

/// An error that was returned from the database.
pub trait DatabaseError: 'static + Send + Sync + StdError {
    /// The primary, human-readable error message.
    fn message(&self) -> &str;

    /// The (SQLSTATE) code for the error.
    fn code(&self) -> Option<Cow<'_, str>> {
        None
    }

    /// The line and column in the executed SQL where the error occurred,
    /// if applicable and supported by the database.
    ///
    /// ### Note
    /// This may return an incorrect result if the database server disagrees with Rust
    /// on the definition of a "character", i.e. a Unicode scalar value. This position should not
    /// be considered authoritative.
    ///
    /// This also may not be returned or made readily available by every database flavor.
    ///
    /// For example, MySQL and MariaDB do not include the error position as a specific field
    /// in the `ERR_PACKET` structure; the line number that appears in the error message is part
    /// of the message string generated by the database server.
    ///
    /// SQLx does not attempt to parse the line number from the message string,
    /// as we cannot assume that the exact message format is a stable part of the API contract.
    fn position(&self) -> Option<ErrorPosition> {
        None
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static);

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static);

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static>;

    #[doc(hidden)]
    fn is_transient_in_connect_phase(&self) -> bool {
        false
    }

    /// Returns the name of the constraint that triggered the error, if applicable.
    /// If the error was caused by a conflict of a unique index, this will be the index name.
    ///
    /// ### Note
    /// Currently only populated by the Postgres driver.
    fn constraint(&self) -> Option<&str> {
        None
    }

    /// Returns the name of the table that was affected by the error, if applicable.
    ///
    /// ### Note
    /// Currently only populated by the Postgres driver.
    fn table(&self) -> Option<&str> {
        None
    }

    /// Returns the kind of the error, if supported.
    ///
    /// ### Note
    /// Not all back-ends behave the same when reporting the error code.
    fn kind(&self) -> ErrorKind;

    /// Returns whether the error kind is a violation of a unique/primary key constraint.
    fn is_unique_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::UniqueViolation)
    }

    /// Returns whether the error kind is a violation of a foreign key.
    fn is_foreign_key_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::ForeignKeyViolation)
    }

    /// Returns whether the error kind is a violation of a check.
    fn is_check_violation(&self) -> bool {
        matches!(self.kind(), ErrorKind::CheckViolation)
    }
}

impl dyn DatabaseError {
    /// Downcast a reference to this generic database error to a specific
    /// database error type.
    ///
    /// # Panics
    ///
    /// Panics if the database error type is not `E`. This is a deliberate contrast from
    /// `Error::downcast_ref` which returns `Option<&E>`. In normal usage, you should know the
    /// specific error type. In other cases, use `try_downcast_ref`.
    pub fn downcast_ref<E: DatabaseError>(&self) -> &E {
        self.try_downcast_ref().unwrap_or_else(|| {
            panic!("downcast to wrong DatabaseError type; original error: {self}")
        })
    }

    /// Downcast this generic database error to a specific database error type.
    ///
    /// # Panics
    ///
    /// Panics if the database error type is not `E`. This is a deliberate contrast from
    /// `Error::downcast` which returns `Option<E>`. In normal usage, you should know the
    /// specific error type. In other cases, use `try_downcast`.
    pub fn downcast<E: DatabaseError>(self: Box<Self>) -> Box<E> {
        self.try_downcast()
            .unwrap_or_else(|e| panic!("downcast to wrong DatabaseError type; original error: {e}"))
    }

    /// Downcast a reference to this generic database error to a specific
    /// database error type.
    #[inline]
    pub fn try_downcast_ref<E: DatabaseError>(&self) -> Option<&E> {
        self.as_error().downcast_ref()
    }

    /// Downcast this generic database error to a specific database error type.
    #[inline]
    pub fn try_downcast<E: DatabaseError>(self: Box<Self>) -> Result<Box<E>, Box<Self>> {
        if self.as_error().is::<E>() {
            Ok(self.into_error().downcast().unwrap())
        } else {
            Err(self)
        }
    }
}

impl<E> From<E> for Error
where
    E: DatabaseError,
{
    #[inline]
    fn from(error: E) -> Self {
        Error::Database(Box::new(error))
    }
}

#[cfg(feature = "migrate")]
impl From<crate::migrate::MigrateError> for Error {
    #[inline]
    fn from(error: crate::migrate::MigrateError) -> Self {
        Error::Migrate(Box::new(error))
    }
}

/// Format an error message as a `Protocol` error
#[macro_export]
macro_rules! err_protocol {
    ($expr:expr) => {
        $crate::error::Error::Protocol($expr.into())
    };

    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::Protocol(format!($fmt, $($arg)*))
    };
}

/// Details the position in an SQL string where the server says an error occurred.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ErrorPosition {
    /// The byte offset where the error occurred.
    pub byte_offset: usize,
    /// The character (Unicode scalar value) offset where the error occurred.
    pub char_offset: usize,
    /// The line number (1-based) in the string.
    pub line: usize,
    /// The column position (1-based) in the string.
    pub column: usize,
}

/// The character basis for an error position. Used with [`ErrorPosition`].
#[derive(Debug)]
pub enum PositionBasis {
    /// A zero-based byte offset.
    ByteOffset(usize),
    /// A zero-based character index.
    CharIndex(usize),
    /// A 1-based character position.
    CharPos(usize),
}

impl ErrorPosition {
    /// Given a query string and a character basis (byte offset, 0-based index or 1-based position),
    /// return the line and column.
    ///
    /// Returns `None` if the character basis is out-of-bounds,
    /// does not lie on a character boundary (byte offsets only),
    /// or overflows `usize`.
    ///
    /// ### Note
    /// This assumes that Rust and the database server agree on the definition of "character",
    /// i.e. a Unicode scalar value.
    pub fn find(query: &str, basis: PositionBasis) -> Option<ErrorPosition> {
        let mut pos = ErrorPosition { byte_offset: 0, char_offset: 0, line: 1, column: 1 };

        for (char_idx, (byte_idx, ch)) in query.char_indices().enumerate() {
            pos.byte_offset = byte_idx;
            pos.char_offset = char_idx;

            // Note: since line and column are 1-based,
            // we technically don't want to advance until the top of the next loop.
            if pos.basis_reached(&basis) {
                return Some(pos);
            }

            if ch == '\n' {
                pos.line = pos.line.checked_add(1)?;
                pos.column = 1;
            } else {
                pos.column = pos.column.checked_add(1)?;
            }
        }

        // Check if the end of the string matches our basis.
        pos.byte_offset = query.len();
        pos.char_offset = pos.char_offset.checked_add(1)?;

        pos.basis_reached(&basis).then_some(pos)
    }

    fn basis_reached(&self, basis: &PositionBasis) -> bool {
        match *basis {
            PositionBasis::ByteOffset(offset) => {
                self.byte_offset == offset
            }
            PositionBasis::CharIndex(char_idx) => {
                self.char_offset == char_idx
            }
            PositionBasis::CharPos(char_pos) => {
                self.char_offset.checked_add(1) == Some(char_pos)
            }
        }
    }
}

#[test]
fn test_error_position() {
    assert_eq!(
        ErrorPosition::find(
            "SELECT foo",
            PositionBasis::CharPos(8),
        ),
        Some(ErrorPosition {
            byte_offset: 7,
            char_offset: 7,
            line: 1,
            column: 8
        })
    );

    assert_eq!(
        ErrorPosition::find(
            "SELECT foo\nbar FROM baz",
            PositionBasis::CharPos(16),
        ),
        Some(ErrorPosition {
            byte_offset: 16,
            char_offset: 16,
            line: 2,
            column: 5
        })
    );

    assert_eq!(
        ErrorPosition::find(
            "SELECT foo\r\nbar FROM baz",
            PositionBasis::CharPos(17)
        ),
        Some(ErrorPosition {
            byte_offset: 16,
            char_offset: 16,
            line: 2,
            column: 5
        })
    );

    assert_eq!(
        ErrorPosition::find(
            "SELECT foo\r\nbar FROM baz",
            PositionBasis::CharPos(27)
        ),
        None
    );
}
