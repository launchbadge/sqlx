use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::BufStream;
use sqlx_core::{Connection, DefaultRuntime, Runtime};

use crate::protocol::Capabilities;
use crate::{MySql, MySqlConnectOptions};

#[cfg(any(feature = "async", feature = "blocking"))]
mod connect;

#[cfg(any(feature = "async", feature = "blocking"))]
mod stream;

#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnection<Rt = DefaultRuntime>
where
    Rt: Runtime,
{
    stream: BufStream<Rt::TcpStream>,
    connection_id: u32,

    // the capability flags are used by the client and server to indicate which
    // features they support and want to use.
    capabilities: Capabilities,

    // the sequence-id is incremented with each packet and may wrap around. It starts at 0 and is
    // reset to 0 when a new command begins in the Command Phase.
    sequence_id: u8,
}

impl<Rt> MySqlConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(any(feature = "async", feature = "blocking"))]
    pub(crate) fn new(stream: Rt::TcpStream) -> Self {
        Self {
            stream: BufStream::with_capacity(stream, 4096, 1024),
            connection_id: 0,
            sequence_id: 0,
            capabilities: Capabilities::PROTOCOL_41 | Capabilities::LONG_PASSWORD
                | Capabilities::LONG_FLAG
                | Capabilities::IGNORE_SPACE
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                // | Capabilities::MULTI_STATEMENTS
                // | Capabilities::MULTI_RESULTS
                // | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                // | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
                // | Capabilities::SESSION_TRACK
                | Capabilities::DEPRECATE_EOF,
        }
    }
}

impl<Rt> Debug for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnection").finish()
    }
}

impl<Rt> Connection<Rt> for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    type Database = MySql;

    type Options = MySqlConnectOptions<Rt>;

    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        unimplemented!()
    }

    #[cfg(feature = "async")]
    fn ping(&mut self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        unimplemented!()
    }
}

#[cfg(feature = "blocking")]
impl<Rt> sqlx_core::blocking::Connection<Rt> for MySqlConnection<Rt>
where
    Rt: sqlx_core::blocking::Runtime,
    <Rt as Runtime>::TcpStream: std::io::Read + std::io::Write,
{
    fn close(self) -> sqlx_core::Result<()> {
        unimplemented!()
    }

    fn ping(&mut self) -> sqlx_core::Result<()> {
        unimplemented!()
    }
}
