use crate::{PgConnectOptions, PgConnection};
use futures_core::future::BoxFuture;
use sqlx_core::error::Error;
use sqlx_core::options::ConnectOptions;

impl ConnectOptions for PgConnectOptions {
    type Connection = PgConnection;

    #[inline]
    fn connect(&self) -> BoxFuture<'_, Result<PgConnection, Error>> {
        Box::pin(PgConnection::connect(self))
    }
}
