use std::io;

use super::{Connection, Runtime};
use crate::DefaultRuntime;

/// Options which can be used to configure how a SQL connection is opened.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`ConnectOptions`][crate::ConnectOptions].
///
#[allow(clippy::module_name_repetitions)]
pub trait ConnectOptions<Rt = DefaultRuntime>: crate::ConnectOptions<Rt>
where
    Rt: Runtime,
    <Rt as crate::Runtime>::TcpStream: io::Read + io::Write,
    Self::Connection: crate::Connection<Rt, Options = Self> + Connection<Rt>,
{
    /// Establish a connection to the database.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`connect()`][crate::ConnectOptions::connect].
    ///
    fn connect(&self) -> crate::Result<Self::Connection>
    where
        Self::Connection: Sized;
}
