use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::mssql::{MssqlConnectOptions, MssqlConnection};
use futures_core::future::BoxFuture;

impl ConnectOptions for MssqlConnectOptions {
    type Connection = MssqlConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(MssqlConnection::establish(self))
    }
}
