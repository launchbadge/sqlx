use crate::blocking::{ConnectOptions, Runtime};
use crate::mysql::{MySqlConnectOptions, MySqlConnection};
use crate::{Blocking, Result};

impl MySqlConnectOptions<Blocking> {
    /// Open a new database connection with the configured connection options.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`connect`](#method.connect).
    ///
    /// Implemented with [`ConnectOptions::connect`].
    #[inline]
    pub fn connect(&self) -> Result<MySqlConnection<Blocking>> {
        <sqlx_mysql::MySqlConnectOptions<Blocking> as ConnectOptions<Blocking>>::connect(&self.0)
            .map(MySqlConnection::<Blocking>)
    }
}

impl<Rt: Runtime> ConnectOptions<Rt> for MySqlConnectOptions<Rt> {
    #[inline]
    fn connect(&self) -> Result<Self::Connection>
    where
        Self::Connection: Sized,
    {
        <sqlx_mysql::MySqlConnectOptions<Rt> as ConnectOptions<Rt>>::connect(&self.0)
            .map(MySqlConnection::<Rt>)
    }
}
