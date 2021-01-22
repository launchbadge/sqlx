use crate::blocking::{ConnectOptions, Runtime};
use crate::postgres::{PostgresConnectOptions, PostgresConnection};
use crate::{Blocking, Result};

impl PostgresConnectOptions<Blocking> {
    /// Open a new database connection with the configured connection options.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`connect`](#method.connect).
    ///
    /// Implemented with [`ConnectOptions::connect`].
    #[inline]
    pub fn connect(&self) -> Result<PostgresConnection<Blocking>> {
        <sqlx_postgres::PostgresConnectOptions<Blocking> as ConnectOptions<Blocking>>::connect(&self.0)
            .map(PostgresConnection::<Blocking>)
    }
}

impl<Rt: Runtime> ConnectOptions<Rt> for PostgresConnectOptions<Rt> {
    #[inline]
    fn connect(&self) -> Result<Self::Connection>
    where
        Self::Connection: Sized,
    {
        <sqlx_postgres::PostgresConnectOptions<Rt> as ConnectOptions<Rt>>::connect(&self.0)
            .map(PostgresConnection::<Rt>)
    }
}
