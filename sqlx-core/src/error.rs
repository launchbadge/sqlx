//! Error<DB>and Result types.

use crate::database::Database;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::io;

/// A specialized `Result` type for SQLx.
pub type Result<DB, T> = std::result::Result<T, Error<DB>>;

/// A generic error that represents all the ways a method can fail inside of SQLx.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error<DB: Database> {
    /// Error<DB>communicating with the database.
    Io(io::Error),

    /// Connection URL was malformed.
    UrlParse(url::ParseError),

    /// An error was returned by the database.
    Database(Box<DB::Error>),

    /// No row was returned during [`Map::fetch_one`] or [`QueryAs::fetch_one`].
    RowNotFound,

    /// Column was not found by name in a Row (during [`Row::get`]).
    ColumnNotFound(Box<str>),

    /// Column index was out of bounds (e.g., asking for column 4 in a 2-column row).
    ColumnIndexOutOfBounds { index: usize, len: usize },

    /// Unexpected or invalid data was encountered. This would indicate that we received
    /// data that we were not expecting or it was in a format we did not understand. This
    /// generally means either there is a programming error in a SQLx driver or
    /// something with the connection or the database database itself is corrupted.
    ///
    /// Context is provided by the included error message.
    Protocol(Box<str>),

    /// A [Pool::acquire] timed out due to connections not becoming available or
    /// because another task encountered too many errors while trying to open a new connection.
    PoolTimedOut(Option<Box<dyn StdError + Send + Sync>>),

    /// [Pool::close] was called while we were waiting in [Pool::acquire].
    PoolClosed,

    /// An error occurred while attempting to setup TLS.
    /// This should only be returned from an explicit ask for TLS.
    Tls(Box<dyn StdError + Send + Sync>),

    /// An error occurred decoding data received from the database.
    Decode(Box<dyn StdError + Send + Sync>),
}

impl<DB: Database> Error<DB> {
    #[allow(dead_code)]
    pub(crate) fn decode<E>(err: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::<DB>::Decode(err.into())
    }
}

impl<DB: Database + Debug> StdError for Error<DB> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::UrlParse(error) => Some(error),
            Error::PoolTimedOut(Some(error)) => Some(&**error),
            Error::Decode(error) => Some(&**error),
            Error::Tls(error) => Some(&**error),

            _ => None,
        }
    }
}

impl<DB: Database> Display for Error<DB> {
    // IntellijRust does not understand that [non_exhaustive] applies only for downstream crates
    // noinspection RsMatchCheck
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{}", error),

            Error::UrlParse(error) => write!(f, "{}", error),

            Error::Decode(error) => write!(f, "{}", error),

            Error::Database(error) => Display::fmt(error, f),

            Error::RowNotFound => f.write_str("found no row when we expected at least one"),

            Error::ColumnNotFound(ref name) => {
                write!(f, "no column found with the name {:?}", name)
            }

            Error::ColumnIndexOutOfBounds { index, len } => write!(
                f,
                "column index out of bounds: there are {} columns but the index is {}",
                len, index
            ),

            Error::Protocol(ref err) => f.write_str(err),

            Error::PoolTimedOut(Some(ref err)) => {
                write!(f, "timed out while waiting for an open connection: {}", err)
            }

            Error::PoolTimedOut(None) => {
                write!(f, "timed out while waiting for an open connection")
            }

            Error::PoolClosed => f.write_str("attempted to acquire a connection on a closed pool"),

            Error::Tls(ref err) => write!(f, "error during TLS upgrade: {}", err),
        }
    }
}

impl<DB: Database> From<io::Error> for Error<DB> {
    #[inline]
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<DB: Database> From<io::ErrorKind> for Error<DB> {
    #[inline]
    fn from(err: io::ErrorKind) -> Self {
        Error::Io(err.into())
    }
}

impl<DB: Database> From<url::ParseError> for Error<DB> {
    #[inline]
    fn from(err: url::ParseError) -> Self {
        Error::UrlParse(err)
    }
}

impl<DB: Database> From<ProtocolError<'_>> for Error<DB> {
    #[inline]
    fn from(err: ProtocolError) -> Self {
        Error::Protocol(err.args.to_string().into_boxed_str())
    }
}

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
impl<DB: Database> From<async_native_tls::Error> for Error<DB> {
    #[inline]
    fn from(err: async_native_tls::Error) -> Self {
        Error::Tls(err.into())
    }
}

impl<DB: Database> From<TlsError<'_>> for Error<DB> {
    #[inline]
    fn from(err: TlsError<'_>) -> Self {
        Error::Tls(err.args.to_string().into())
    }
}

/// An error that was returned by the database.
pub trait DatabaseError: StdError + Send + Sync {
    /// The primary, human-readable error message.
    fn message(&self) -> &str;

    /// The (SQLSTATE) code for the error.
    fn code(&self) -> Option<&str> {
        None
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        None
    }

    fn column_name(&self) -> Option<&str> {
        None
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }
}

/// Used by the `protocol_error!()` macro for a lazily evaluated conversion to
/// `crate::Error::Protocol` so we can use the macro with `.ok_or()` without Clippy complaining.
pub(crate) struct ProtocolError<'a> {
    pub args: fmt::Arguments<'a>,
}

#[allow(unused_macros)]
macro_rules! protocol_err (
    ($($args:tt)*) => {
        $crate::error::ProtocolError { args: format_args!($($args)*) }
    }
);

pub(crate) struct TlsError<'a> {
    pub args: fmt::Arguments<'a>,
}

#[allow(unused_macros)]
macro_rules! tls_err {
    ($($args:tt)*) => { crate::error::TlsError { args: format_args!($($args)*)} };
}

#[allow(unused_macros)]
macro_rules! decode_err {
    ($s:literal, $($args:tt)*) => {
        crate::Error::Decode(format!($s, $($args)*).into())
    };

    ($expr:expr) => {
        crate::Error::decode($expr)
    };
}

/// An unexpected `NULL` was encountered during decoding.
///
/// Returned from `Row::get` if the value from the database is `NULL`
/// and you are not decoding into an `Option`.
#[derive(Debug, Clone, Copy)]
pub struct UnexpectedNullError;

impl Display for UnexpectedNullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unexpected null; try decoding as an `Option`")
    }
}

impl StdError for UnexpectedNullError {}
