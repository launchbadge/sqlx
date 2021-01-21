use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;

#[cfg(feature = "async")]
use futures_util::future::{BoxFuture, FutureExt};

use crate::mysql::MySqlConnection;
#[cfg(feature = "async")]
use crate::Async;
use crate::{ConnectOptions, DefaultRuntime, Error, Result, Runtime};

mod builder;
mod getters;

/// Options which can be used to configure how a MySQL connection is opened.
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnectOptions<Rt: Runtime = DefaultRuntime>(
    pub(super) sqlx_mysql::MySqlConnectOptions<Rt>,
);

impl<Rt: Runtime> MySqlConnectOptions<Rt> {
    /// Creates a default set of connection options.
    ///
    /// Implemented with [`Default`](#impl-Default).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses connection options from a connection URL.
    ///
    /// ```text
    /// mysql://[[user[:password]@]host][/database][?properties]
    /// ```
    ///
    /// Implemented with [`FromStr`](#impl-FromStr).
    ///
    #[inline]
    pub fn parse(url: &str) -> Result<Self> {
        Ok(Self(url.parse()?))
    }
}

#[cfg(feature = "async")]
impl<Rt: Async> MySqlConnectOptions<Rt> {
    /// Open a new database connection with the configured connection options.
    ///
    /// Implemented with [`ConnectOptions::connect`].
    #[inline]
    pub async fn connect(&self) -> Result<MySqlConnection<Rt>> {
        <sqlx_mysql::MySqlConnectOptions<Rt> as ConnectOptions<Rt>>::connect(&self.0)
            .await
            .map(MySqlConnection)
    }
}

impl<Rt: Runtime> ConnectOptions<Rt> for MySqlConnectOptions<Rt> {
    type Connection = MySqlConnection<Rt>;

    #[cfg(feature = "async")]
    #[inline]
    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection>>
    where
        Self::Connection: Sized,
        Rt: Async,
    {
        self.connect().boxed()
    }
}

impl<Rt: Runtime> Debug for MySqlConnectOptions<Rt> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<Rt: Runtime> Default for MySqlConnectOptions<Rt> {
    fn default() -> Self {
        Self(sqlx_mysql::MySqlConnectOptions::<Rt>::default())
    }
}

impl<Rt: Runtime> Clone for MySqlConnectOptions<Rt> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Rt: Runtime> FromStr for MySqlConnectOptions<Rt> {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self> {
        Ok(Self(url.parse()?))
    }
}
