#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

use crate::{Close, Connect, Database, Runtime};

/// A single connection (also known as a session) with a specific database.
///
/// With a client/server model, this is equivalent to a network connection
/// to the server.
///
/// SQL statements will be executed and results returned within the context
/// of this single SQL connection.
///
// for<'a> &'a mut Rt::TcpStream: crate::io::Stream<'a>,
pub trait Connection<Rt>: 'static + Send + Connect<Rt> + Close<Rt>
where
    Rt: Runtime,
{
    type Database: Database;

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
        Rt: crate::Async;
}
