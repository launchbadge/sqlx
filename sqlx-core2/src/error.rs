//! Errors produced by SQLx.

use std::error::Error as StdError;

/// A boxed alias of [`std::error::Error`] used in the variants of [`Error`]
/// to accept unknown error types.
///
/// Ideally, Rust would provide an error primitive such as `std::error` or a [boxed alias itself](https://github.com/rust-lang/rfcs/pull/2820).
///
pub type BoxStdError = Box<dyn StdError + Send + Sync>;

// TODO: #RowNotFound
// TODO: #ColumnIndexOutOfBounds
// TODO: #ColumnNotFound
// TODO: #ColumnDecode -> #ColumnFromValue
// TODO: #Decode -> #FromValue
// TODO: #ToValue
// TODO: #PoolTimedOut
// TODO: #PoolClosed
// TODO: #Migrate

/// Represents all the ways a method can fail within SQLx.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error occurred while parsing a connection string or otherwise resolving configuration.
    #[error("error with configuration: {0}")]
    Configuration(#[source] BoxStdError),

    /// Error communicating with the database server.
    #[error("error communicating with the server: {0}")]
    Network(#[from] std::io::Error),

    /// Unexpected or invalid data encountered while communicating with the database.
    ///
    /// This should indicate there is a programming error in a SQLx driver or there
    /// is something corrupted with the connection to the database itself.
    #[error("encountered unexpected or invalid data: {0}")]
    Protocol(#[source] BoxStdError),

    /// Invalid SQL query or arguments.
    ///
    /// Typically we catch these kinds of errors at compile-time with Rust's type system. An example
    /// of when it can still occur is a query string that is too large. For instance, in PostgreSQL,
    /// query strings have a maximum size of `i32::MAX`.
    #[error("{0}")]
    Query(#[source] BoxStdError),

    /// Error occurred while attempting to establish a TLS connection.
    #[error("error occurred while attempting to establish a TLS connection: {0}")]
    Tls(#[source] BoxStdError),
}

#[doc(hidden)]
impl Error {
    #[inline]
    pub fn protocol(err: impl StdError + Send + Sync + 'static) -> Self {
        Error::Protocol(err.into())
    }

    #[inline]
    pub fn protocol_msg(msg: impl Into<String>) -> Self {
        Error::Protocol(msg.into().into())
    }

    #[inline]
    pub fn configuration(err: impl StdError + Send + Sync + 'static) -> Self {
        Error::Configuration(err.into())
    }

    #[inline]
    pub fn configuration_msg(msg: impl Into<String>) -> Self {
        Error::Configuration(msg.into().into())
    }

    #[inline]
    pub fn tls(err: impl StdError + Send + Sync + 'static) -> Self {
        Error::Tls(err.into())
    }
}
