//! Provides the [`ConnectOptions`] trait for configuring a new connection.

use crate::connection::Connection;
use crate::error::Error;
use futures_core::future::BoxFuture;
use std::fmt::Debug;
use std::str::FromStr;

/// Connection options for configuring a new connection.
///
/// Can be parsed from a semi-universal connection URI format of the form:
///
/// ```text
/// driver://user:pass@host:port/database?param1=value1&param2=value2
/// ```
///
pub trait ConnectOptions: 'static + Send + Sync + FromStr<Err = Error> + Debug {
    type Connection: Connection + ?Sized;

    /// Establish a new database connection with the options specified by `self`.
    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized;
}
