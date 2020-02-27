//! Error and Result types.

use crate::decode::DecodeError;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::io;

/// A specialized `Result` type for SQLx.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A generic error that represents all the ways a method can fail inside of SQLx.
#[derive(Debug)]
pub enum Error {
    /// Error communicating with the database.
    Io(io::Error),

    /// Connection URL was malformed.
    UrlParse(url::ParseError),

    /// An error was returned by the database.
    Database(Box<dyn DatabaseError + Send + Sync>),

    /// No rows were returned by a query that expected to return at least one row.
    NotFound,

    /// More than one row was returned by a query that expected to return exactly one row.
    FoundMoreThanOne,

    /// Column was not found by name in a Row (during [Row::try_get]).
    ColumnNotFound(Box<str>),

    /// Column index was out of bounds (e.g., asking for column 4 in a 2-column row).
    ColumnIndexOutOfBounds {
        index: usize,
        len: usize,
    },

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

    /// An error occurred during a TLS upgrade.
    TlsUpgrade(Box<dyn StdError + Send + Sync>),

    Decode(DecodeError),

    // TODO: Remove and replace with `#[non_exhaustive]` when possible
    #[doc(hidden)]
    __Nonexhaustive,
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(error) => Some(error),

            Error::UrlParse(error) => Some(error),

            Error::PoolTimedOut(Some(error)) => Some(&**error),

            Error::Decode(DecodeError::Other(error)) => Some(&**error),

            Error::TlsUpgrade(error) => Some(&**error),

            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{}", error),

            Error::UrlParse(error) => write!(f, "{}", error),

            Error::Decode(error) => write!(f, "{}", error),

            Error::Database(error) => Display::fmt(error, f),

            Error::NotFound => f.write_str("found no rows when we expected at least one"),

            Error::ColumnNotFound(ref name) => {
                write!(f, "no column found with the name {:?}", name)
            }

            Error::ColumnIndexOutOfBounds { index, len } => write!(
                f,
                "column index out of bounds: there are {} columns but the index is {}",
                len, index
            ),

            Error::FoundMoreThanOne => {
                f.write_str("found more than one row when we expected exactly one")
            }

            Error::Protocol(ref err) => f.write_str(err),

            Error::PoolTimedOut(Some(ref err)) => {
                write!(f, "timed out while waiting for an open connection: {}", err)
            }

            Error::PoolTimedOut(None) => {
                write!(f, "timed out while waiting for an open connection")
            }

            Error::PoolClosed => f.write_str("attempted to acquire a connection on a closed pool"),

            Error::TlsUpgrade(ref err) => write!(f, "error during TLS upgrade: {}", err),

            Error::__Nonexhaustive => unreachable!(),
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<io::ErrorKind> for Error {
    #[inline]
    fn from(err: io::ErrorKind) -> Self {
        Error::Io(err.into())
    }
}

impl From<DecodeError> for Error {
    #[inline]
    fn from(err: DecodeError) -> Self {
        Error::Decode(err)
    }
}

impl From<url::ParseError> for Error {
    #[inline]
    fn from(err: url::ParseError) -> Self {
        Error::UrlParse(err)
    }
}

impl From<ProtocolError<'_>> for Error {
    #[inline]
    fn from(err: ProtocolError) -> Self {
        Error::Protocol(err.args.to_string().into_boxed_str())
    }
}

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
impl From<async_native_tls::Error> for Error {
    #[inline]
    fn from(err: async_native_tls::Error) -> Self {
        Error::TlsUpgrade(err.into())
    }
}

impl From<TlsError<'_>> for Error {
    #[inline]
    fn from(err: TlsError<'_>) -> Self {
        Error::TlsUpgrade(err.args.to_string().into())
    }
}

impl<T> From<T> for Error
where
    T: 'static + DatabaseError,
{
    #[inline]
    fn from(err: T) -> Self {
        Error::Database(Box::new(err))
    }
}

/// An error that was returned by the database.
pub trait DatabaseError: Display + Debug + Send + Sync {
    /// The primary, human-readable error message.
    fn message(&self) -> &str;

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
macro_rules! impl_fmt_error {
    ($err:ty) => {
        impl std::fmt::Debug for $err {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct("DatabaseError")
                    .field("message", &self.message())
                    .field("details", &self.details())
                    .field("hint", &self.hint())
                    .field("table_name", &self.table_name())
                    .field("column_name", &self.column_name())
                    .field("constraint_name", &self.constraint_name())
                    .finish()
            }
        }

        impl std::fmt::Display for $err {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.pad(self.message())
            }
        }
    };
}
