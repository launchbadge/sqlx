use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display},
    io,
};

pub type Result<T> = std::result::Result<T, Error>;

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
    Database(Box<dyn DbError + Send + Sync>),

    /// No rows were returned by a query expected to return at least one row.
    NotFound,

    // TODO: Remove and replace with `#[non_exhaustive]` when possible
    #[doc(hidden)]
    __Nonexhaustive,
}

// TODO: Forward causes where present
impl StdError for Error {}

// TODO: Don't just forward to debug
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<T> From<T> for Error
where
    T: 'static + DbError,
{
    #[inline]
    fn from(err: T) -> Self {
        Error::Database(Box::new(err))
    }
}

/// An error that was returned by the database backend.
pub trait DbError: Debug + Send + Sync {
    fn message(&self) -> &str;

    // TODO: Expose more error properties
}
