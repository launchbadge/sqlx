use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::postgres::{PgConnectOptions, PgConnection};
use futures_core::future::BoxFuture;

impl ConnectOptions for PgConnectOptions {
    type Connection = PgConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(PgConnection::establish(self))
    }
}
