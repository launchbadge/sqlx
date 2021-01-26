use std::collections::VecDeque;
use std::fmt::{self, Debug, Formatter};

use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Close, Connect, Connection, Runtime};

use crate::protocol::Capabilities;
use crate::stream::MySqlStream;
use crate::{MySql, MySqlConnectOptions};

mod close;
mod command;
mod connect;
mod executor;
mod ping;

/// A single connection (also known as a session) to a MySQL database server.
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnection<Rt>
where
    Rt: Runtime,
{
    stream: MySqlStream<Rt>,
    connection_id: u32,

    // the capability flags are used by the client and server to indicate which
    // features they support and want to use.
    capabilities: Capabilities,

    // queue of commands that are being processed
    // this is what we expect to receive from the server
    // in the case of a future or stream being dropped
    commands: VecDeque<command::Command>,
}

impl<Rt> MySqlConnection<Rt>
where
    Rt: Runtime,
{
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self {
            stream: MySqlStream::new(stream),
            connection_id: 0,
            commands: VecDeque::with_capacity(2),
            capabilities: Capabilities::PROTOCOL_41
                | Capabilities::LONG_PASSWORD
                | Capabilities::LONG_FLAG
                | Capabilities::IGNORE_SPACE
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                | Capabilities::MULTI_STATEMENTS
                | Capabilities::MULTI_RESULTS
                | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
                | Capabilities::SESSION_TRACK
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

    #[cfg(feature = "async")]
    fn ping(&mut self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(self.ping_async())
    }
}

impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
    type Options = MySqlConnectOptions<Rt>;

    #[cfg(feature = "async")]
    fn connect(url: &str) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<Self>>
    where
        Self: Sized,
        Rt: sqlx_core::Async,
    {
        use sqlx_core::ConnectOptions;

        let options = url.parse::<Self::Options>();
        Box::pin(async move { options?.connect().await })
    }
}

impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(self.close_async())
    }
}

#[cfg(feature = "blocking")]
mod blocking {
    use sqlx_core::blocking::{Close, Connect, Connection, Runtime};

    use super::{MySqlConnectOptions, MySqlConnection};

    impl<Rt: Runtime> Connection<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn ping(&mut self) -> sqlx_core::Result<()> {
            self.ping_blocking()
        }
    }

    impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn connect(url: &str) -> sqlx_core::Result<Self>
        where
            Self: Sized,
        {
            Self::connect_blocking(&url.parse::<MySqlConnectOptions<Rt>>()?)
        }
    }

    impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn close(self) -> sqlx_core::Result<()> {
            self.close_blocking()
        }
    }
}
