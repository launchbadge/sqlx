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
    pub fn connect(&self) -> Result<MySqlConnectOptions<Blocking>> {
        <sqlx_mysql::MySqlConnectOptions<Rt> as ConnectOptions<Rt>>::connect(&self.0)
            .map(MySqlConnectOptions::<Blocking>)
    }
}

impl<Rt> ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn connect(&self) -> Result<Self::Connection>
    where
        Self::Connection: Sized,
    {
        self.connect()
    }
}
