use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use either::Either;

use crate::decode::Error as DecodeError;
use crate::encode::Error as EncodeError;

mod client;
mod database;

pub use client::ClientError;
pub use database::DatabaseError;

use crate::arguments::ArgumentIndex;
use crate::Column;

/// Specialized `Result` type returned from fallible methods within SQLx.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type returned for all methods in SQLX.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The database URL is malformed or contains invalid or unsupported
    /// values for one or more options; a value of [`ConnectOptions`] failed
    /// to be parsed.
    ConnectOptions { message: Cow<'static, str>, source: Option<Box<dyn StdError + Send + Sync>> },

    /// An error that was returned from the database, normally from the
    /// execution of a SQL command.
    ///
    Database(Box<dyn DatabaseError>),

    /// An error was identified on the client from the result of interacting
    /// with the database.
    ///
    Client(Box<dyn ClientError>),

    /// An IO error returned while reading or writing a socket attached
    /// to the database server.
    ///
    /// Only applicable if the database driver connects to a remote database
    /// server.
    ///
    Network(std::io::Error),

    /// No rows returned by a query required to return at least one row.
    ///
    /// Returned by `fetch_one` when no rows were returned from
    /// the query. Use `fetch_optional` to return `None` instead
    /// of signaling an error.
    ///
    RowNotFound,

    /// An attempt to act on a closed connection or pool.
    ///
    /// A connection will close itself on an unrecoverable error in the
    /// connection (implementation bugs, faulty network, etc.). If the error
    /// was ignored and the connection is used again, it will
    /// return `Error::Closed`.
    ///
    /// A pool will return `Error::Closed` from `Pool::acquire` if `Pool::close`
    /// was called before `acquire` received a connection.
    ///
    Closed,

    /// An error occurred decoding a SQL value from the database.
    Decode(DecodeError),

    /// An error occurred encoding a value to be sent to the database.
    Encode(EncodeError),

    /// An attempt to access a column by index past the end of the row.
    ColumnIndexOutOfBounds { index: usize, len: usize },

    /// An attempt to access a column by name where no such column is
    /// present in the row.
    ColumnNotFound { name: Box<str> },

    /// An error occurred decoding a SQL value of a specific column
    /// from the database.
    ColumnDecode { column_index: usize, column_name: Box<str>, source: DecodeError },

    /// An error occurred encoding a value for a specific parameter to
    /// be sent to the database.
    ParameterEncode { parameter: ArgumentIndex<'static>, source: EncodeError },

    /// An error occurred while parsing or expanding the generic placeholder syntax in a query.
    Placeholders(crate::placeholders::Error),
}

impl Error {
    #[doc(hidden)]
    pub fn opt(
        message: impl Into<Cow<'static, str>>,
        source: impl Into<Box<dyn StdError + Send + Sync>>,
    ) -> Self {
        Self::ConnectOptions { message: message.into(), source: Some(source.into()) }
    }

    #[doc(hidden)]
    pub fn opt_msg(message: impl Into<Cow<'static, str>>) -> Self {
        Self::ConnectOptions { message: message.into(), source: None }
    }

    #[doc(hidden)]
    pub fn client(err: impl ClientError) -> Self {
        Self::Client(Box::new(err))
    }

    #[doc(hidden)]
    pub fn database(err: impl DatabaseError) -> Self {
        Self::Database(Box::new(err))
    }

    #[doc(hidden)]
    pub fn column_decode(column: &impl Column, source: DecodeError) -> Self {
        crate::Error::ColumnDecode {
            source,
            column_index: column.index(),
            column_name: column.name().to_owned().into_boxed_str(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(source) => write!(f, "{}", source),

            Self::Database(source) => write!(f, "{}", source),

            Self::Client(source) => write!(f, "{}", source),

            Self::ConnectOptions { message, source: None } => {
                write!(f, "{}", message)
            }

            Self::ConnectOptions { message, source: Some(source) } => {
                write!(f, "{}: {}", message, source)
            }

            Self::RowNotFound => {
                f.write_str("No row returned by a query required to return at least one row")
            }

            Self::Closed => f.write_str("Connection or pool is closed"),

            Self::Decode(error) => {
                write!(f, "Decode: {}", error)
            }

            Self::Encode(error) => {
                write!(f, "Encode: {}", error)
            }

            Self::ColumnIndexOutOfBounds { index, len } => {
                write!(
                    f,
                    "Column index out of bounds: the len is {}, but the index is {}",
                    len, index
                )
            }

            Self::ColumnNotFound { name } => {
                write!(f, "No column found for name `{}`", name)
            }

            Self::ColumnDecode { column_index, column_name, source } => {
                if column_name.is_empty() {
                    write!(f, "Decode column {}: {}", column_index, source)
                } else {
                    write!(f, "Decode column {} `{}`: {}", column_index, column_name, source)
                }
            }

            Self::ParameterEncode { parameter, source } => {
                write!(f, "Encode parameter {}: {}", parameter, source)
            }

            Self::Placeholders(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::ConnectOptions { source: Some(source), .. } => Some(&**source),
            Self::Network(source) => Some(source),
            Self::Placeholders(source) => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Network(error)
    }
}

impl From<std::io::ErrorKind> for Error {
    fn from(error: std::io::ErrorKind) -> Self {
        Self::Network(error.into())
    }
}
