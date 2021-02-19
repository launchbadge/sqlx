use std::fmt::{self, Debug, Formatter};

#[cfg(feature = "async")]
use futures_util::future::{BoxFuture, FutureExt};
use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Close, Connect, Connection, Runtime};

use crate::connection::command::CommandQueue;
use crate::protocol::Capabilities;
use crate::stream::MySqlStream;
use crate::{MySql, MySqlConnectOptions};

#[macro_use]
mod flush;

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
    closed: bool,

    // the capability flags are used by the client and server to indicate which
    // features they support and want to use.
    capabilities: Capabilities,

    // queue of commands that are being processed
    // this is what we expect to receive from the server
    // in the case of a future or stream being dropped
    commands: CommandQueue,
}

impl<Rt> MySqlConnection<Rt>
where
    Rt: Runtime,
{
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self {
            stream: MySqlStream::new(stream),
            connection_id: 0,
            closed: false,
            commands: CommandQueue::new(),
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
    fn ping(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(self.ping_async())
    }
}

impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
    type Options = MySqlConnectOptions;

    #[cfg(feature = "async")]
    fn connect_with(options: &MySqlConnectOptions) -> BoxFuture<'_, sqlx_core::Result<Self>>
    where
        Self: Sized,
        Rt: sqlx_core::Async,
    {
        MySqlConnection::connect_async(options).boxed()
    }
}

impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    fn close(mut self) -> BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(async move {
            self.stream.close_async().await?;

            Ok(())
        })
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
        fn connect_with(options: &MySqlConnectOptions) -> sqlx_core::Result<Self>
        where
            Self: Sized,
        {
            Self::connect_blocking(options)
        }
    }

    impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
        #[inline]
        fn close(mut self) -> sqlx_core::Result<()> {
            self.stream.close_blocking()
        }
    }
}
