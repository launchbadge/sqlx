use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display},
    io,
};

use async_std::future::TimeoutError;

/// A convenient Result instantiation appropriate for SQLx.
pub type Result<T> = std::result::Result<T, Error>;

/// A generic error that represents all the ways a method can fail inside of SQLx.
#[derive(Debug)]
pub enum Error {
    /// Error communicating with the database backend.
    ///
    /// Some reasons for this to be caused:
    ///
    ///  - [io::ErrorKind::ConnectionRefused] - Database backend is most likely behind a firewall.
    ///
    ///  - [io::ErrorKind::ConnectionReset] - Database backend dropped the client connection (perhaps from an administrator action).
    Io(io::Error),

    /// An error was returned by the database backend.
    Database(Box<dyn DatabaseError + Send + Sync>),

    /// No rows were returned by a query expected to return at least one row.
    NotFound,

    /// More than one row was returned by a query expected to return exactly one row.
    FoundMoreThanOne,

    /// Unexpected or invalid data was encountered. This would indicate that we received data that we were not
    /// expecting or it was in a format we did not understand. This generally means either there is a programming error in a SQLx driver or
    /// something with the connection or the database backend itself is corrupted.
    ///
    /// Context is provided by the included error message.
    Protocol(Box<str>),

    /// A `Pool::acquire()` timed out due to connections not becoming available or
    /// because another task encountered too many errors while trying to open a new connection.
    TimedOut,

    /// `Pool::close()` was called while we were waiting in `Pool::acquire()`.
    PoolClosed,

    // TODO: Remove and replace with `#[non_exhaustive]` when possible
    #[doc(hidden)]
    __Nonexhaustive,
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(error) => Some(error),

            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{}", error),

            Error::Database(error) => Display::fmt(error, f),

            Error::NotFound => f.write_str("found no rows when we expected at least one"),

            Error::FoundMoreThanOne => {
                f.write_str("found more than one row when we expected exactly one")
            }

            Error::Protocol(ref err) => f.write_str(err),

            Error::TimedOut => f.write_str("timed out while waiting for an open connection"),

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

impl From<TimeoutError> for Error {
    fn from(_: TimeoutError) -> Self {
        Error::TimedOut
    }
}

impl From<ProtocolError<'_>> for Error {
    #[inline]
    fn from(err: ProtocolError) -> Self {
        Error::Protocol(err.args.to_string().into_boxed_str())
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

/// An error that was returned by the database backend.
pub trait DatabaseError: Display + Debug + Send + Sync {
    fn message(&self) -> &str;
}

/// Used by the `protocol_error!()` macro for a lazily evaluated conversion to
/// `crate::Error::Protocol` so we can use the macro with `.ok_or()` without Clippy complaining.
pub(crate) struct ProtocolError<'a> {
    pub args: fmt::Arguments<'a>,
}

macro_rules! protocol_err (
    ($($args:tt)*) => {
        $crate::error::ProtocolError { args: format_args!($($args)*) }
    }
);
