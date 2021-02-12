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
        <Self as ConnectOptions>::connect::<MySqlConnection<Blocking>, Blocking>(self)
    }
}

impl<Rt: Runtime> ConnectOptions for MySqlConnectOptions<Rt> {}
