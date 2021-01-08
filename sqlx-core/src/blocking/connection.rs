use std::io;

use super::{ConnectOptions, Runtime};
use crate::DefaultRuntime;

/// A unique connection (session) with a specific database.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`Connection`][crate::Connection].
///
pub trait Connection<Rt = DefaultRuntime>: crate::Connection<Rt>
where
    Rt: Runtime,
    <Rt as crate::Runtime>::TcpStream: io::Read + io::Write,
    Self::Options: ConnectOptions<Rt>,
{
    /// Establish a new database connection.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`connect()`][crate::Connection::connect].
    ///
    fn connect(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        url.parse::<<Self as crate::Connection<Rt>>::Options>()?.connect()
    }

    /// Explicitly close this database connection.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`close()`][crate::Connection::close].
    ///
    fn close(self) -> crate::Result<()>;

    /// Checks if a connection to the database is still valid.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`ping()`][crate::Connection::ping].
    ///
    fn ping(&mut self) -> crate::Result<()>;
}
