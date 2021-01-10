#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

use crate::{Close, Connect, Database, DefaultRuntime, Runtime};

/// A unique connection (session) with a specific database.
///
/// With a client/server model, this is equivalent to a network connection
/// to the server.
///
/// SQL statements will be executed and results returned within the context
/// of this single SQL connection.
///
pub trait Connection<Rt: Runtime = DefaultRuntime>:
    'static + Send + Connect<Rt> + Close<Rt>
{
    type Database: Database<Rt, Connection = Self>;

    /// Checks if a connection to the database is still valid.
    ///
    /// The method of operation greatly depends on the database driver. In MySQL, there is an
    /// explicit [`COM_PING`](https://dev.mysql.com/doc/internals/en/com-ping.html) command. In
    /// PostgreSQL, `ping` will issue a query consisting of a comment `/* SQLx ping */` which,
    /// in effect, does nothing apart from getting a response from the server.
    ///
    #[cfg(feature = "async")]
    fn ping(&mut self) -> BoxFuture<'_, crate::Result<()>>
    where
        Rt: crate::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin;
}
