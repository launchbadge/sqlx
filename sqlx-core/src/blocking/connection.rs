use std::io;

use super::{Blocking, Close, Connect, ConnectOptions, Runtime};

/// A unique connection (session) with a specific database.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`Connection`][crate::Connection].
///
pub trait Connection<Rt: Runtime = Blocking>:
    crate::Connection<Rt> + Close<Rt> + Connect<Rt>
where
    Rt: Runtime,
    <Rt as crate::Runtime>::TcpStream: io::Read + io::Write,
    Self::Options: ConnectOptions<Rt>,
{
    /// Checks if a connection to the database is still valid.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`ping()`][crate::Connection::ping].
    ///
    fn ping(&mut self) -> crate::Result<()>;
}
