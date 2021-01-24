use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::BufStream;
use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Close, Connect, Connection, Runtime};

use crate::{Postgres, PostgresConnectOptions};

#[macro_use]
mod sasl;

mod close;
mod connect;
mod ping;
mod stream;

/// A single connection (also known as a session) to a PostgreSQL database server.
#[allow(clippy::module_name_repetitions)]
pub struct PostgresConnection<Rt>
where
    Rt: Runtime,
{
    stream: BufStream<Rt, NetStream<Rt>>,

    // process id of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    process_id: u32,

    // secret key of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    secret_key: u32,
}

impl<Rt> PostgresConnection<Rt>
where
    Rt: Runtime,
{
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self { stream: BufStream::with_capacity(stream, 4096, 1024), process_id: 0, secret_key: 0 }
    }
}

impl<Rt> Debug for PostgresConnection<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConnection").finish()
    }
}

impl<Rt> Connection<Rt> for PostgresConnection<Rt>
where
    Rt: Runtime,
{
    type Database = Postgres;

    #[cfg(feature = "async")]
    fn ping(&mut self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        Box::pin(self.ping_async())
    }
}

impl<Rt: Runtime> Connect<Rt> for PostgresConnection<Rt> {
    type Options = PostgresConnectOptions<Rt>;

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

impl<Rt: Runtime> Close<Rt> for PostgresConnection<Rt> {
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

    use super::{PostgresConnectOptions, PostgresConnection};

    impl<Rt: Runtime> Connection<Rt> for PostgresConnection<Rt> {
        #[inline]
        fn ping(&mut self) -> sqlx_core::Result<()> {
            self.ping()
        }
    }

    impl<Rt: Runtime> Connect<Rt> for PostgresConnection<Rt> {
        #[inline]
        fn connect(url: &str) -> sqlx_core::Result<Self>
        where
            Self: Sized,
        {
            Self::connect(&url.parse::<PostgresConnectOptions<Rt>>()?)
        }
    }

    impl<Rt: Runtime> Close<Rt> for PostgresConnection<Rt> {
        #[inline]
        fn close(self) -> sqlx_core::Result<()> {
            self.close()
        }
    }
}
