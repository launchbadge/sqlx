use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display},
    io,
};

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
    ///
    ///  - [io::ErrorKind::InvalidData] - Unexpected or invalid data was encountered. This would indicate that we received data that we were not
    ///         expecting or it was in a format we did not understand. This generally means either there is a programming error in a SQLx driver or
    ///         something with the connection or the database backend itself is corrupted. Additional details are provided along with the
    ///         error.
    ///
    Io(io::Error),

    /// An error was returned by the database backend.
    Database(Box<dyn DatabaseError + Send + Sync>),

    /// No rows were returned by a query expected to return at least one row.
    NotFound,

    /// More than one row was returned by a query expected to return exactly one row.
    FoundMoreThanOne,

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

impl From<InvalidData<'_>> for Error {
    #[inline]
    fn from(err: InvalidData) -> Self {
        Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            err.args.to_string(),
        ))
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

/// Used by the `invalid_data!()` macro for a lazily evaluated conversion to `io::Error`
/// so we can use the macro with `.ok_or()` without Clippy complaining.
pub(crate) struct InvalidData<'a> {
    pub args: fmt::Arguments<'a>,
}
