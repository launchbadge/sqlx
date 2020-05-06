use std::error::Error as StdError;
use std::io;
use std::result::Result as StdResult;

/// A specialized `Result` type for SQLx.
pub type Result<T> = StdResult<T, Error>;

// Convenience type alias for usage within SQLx.
pub(crate) type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Represents all the ways a method can fail within SQLx.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Error occurred while parsing connection options.
    #[error("error occurred while parsing connection options: {0}")]
    ParseConnectOptions(#[source] BoxDynError),

    /// Error returned from the database.
    #[error("error returned from database: {0}")]
    Database(Box<dyn DatabaseError>),

    /// Error communicating with the database backend.
    #[error("error communicating with the server: {0}")]
    Io(#[from] io::Error),

    /// Error occurred while attempting to establish a TLS connection.
    #[error("error occurred while attempting to establish a TLS connection: {0}")]
    Tls(#[from] async_native_tls::Error),

    /// Unexpected or invalid data encountered while communicating with the database.
    ///
    /// This should indicate there is a programming error in a SQLx driver or there
    /// is something corrupted with the connection to the database itself.
    #[error("encountered unexpected or invalid data: {0}")]
    Protocol(String),

    /// No rows returned by a query that expected to return at least one row.
    #[error("no rows returned by a query that expected to return at least one row")]
    RowNotFound,

    /// More than one row returned by a query that expected to return exactly one row.
    #[error("more than one row returned by a query that expected to return exactly one row")]
    FoundMoreThanOneRow,

    /// More than one column returned by a query that expected to return exactly one column.
    #[error("more than one column returned by a query that expected to return exactly one column")]
    FoundMoreThanOneColumn,

    /// Column index was out of bounds.
    #[error("column index out of bounds: the len is {len}, but the index is {index}")]
    ColumnIndexOutOfBounds { index: usize, len: usize },

    /// No column found for name.
    #[error("no column found for name: {0}")]
    ColumnNotFound(String),

    /// Error occurred while decoding a value received from the database.
    #[error("error occurred while decoding column {index}: {cause}")]
    Decode {
        index: String,

        #[source]
        cause: BoxDynError,
    },
}

/// An error that was returned from the database.
pub trait DatabaseError: 'static + Send + Sync + StdError {
    // TODO: Re-expose common error details
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

// Format an error message as a `Protocol` error
macro_rules! err_protocol {
    ($expr:expr) => {
        $crate::error::Error::Protocol($expr.into())
    };

    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::Protocol(format!($fmt, $($arg)*))
    };
}
