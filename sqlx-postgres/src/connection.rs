use std::fmt::{self, Debug, Formatter};

#[cfg(feature = "async")]
use futures_util::future::{BoxFuture, FutureExt, TryFutureExt};
use sqlx_core::{Close, Connect, Connection, Runtime};

use crate::stream::PgStream;
use crate::Postgres;

/// A single connection (also known as a session) to a
/// PostgreSQL database server.
pub struct PgConnection<Rt: Runtime> {
    stream: PgStream<Rt>,

    // process id of this backend
    // can be used to send cancel requests
    #[allow(dead_code)]
    process_id: u32,

    // secret key of this backend
    // can be used to send cancel requests
    #[allow(dead_code)]
    secret_key: u32,
}

impl<Rt> Debug for PgConnection<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnection").finish()
    }
}

impl<Rt: Runtime> PgConnection<Rt> {
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self { stream: PgStream::new(stream), process_id: 0, secret_key: 0 }
    }
}

impl<Rt: Runtime> Connection<Rt> for PgConnection<Rt> {
    type Database = Postgres;

    #[cfg(feature = "async")]
    fn ping(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        todo!()
    }

    #[cfg(feature = "async")]
    fn describe<'x, 'e, 'q>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'x, sqlx_core::Result<sqlx_core::Describe<Postgres>>>
    where
        Rt: sqlx_core::Async,
        'e: 'x,
        'q: 'x,
    {
        todo!()
    }
}

impl<Rt: Runtime> Connect<Rt> for PgConnection<Rt> {
    type Options = PostgresConnectOptions;

    #[cfg(feature = "async")]
    fn connect_with(options: &PostgresConnectOptions) -> BoxFuture<'_, sqlx_core::Result<Self>>
    where
        Self: Sized,
        Rt: sqlx_core::Async,
    {
        PgConnection::connect_async(options).boxed()
    }
}

impl<Rt: Runtime> Close<Rt> for PgConnection<Rt> {
    #[cfg(feature = "async")]
    fn close(mut self) -> BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        todo!()
    }
}
