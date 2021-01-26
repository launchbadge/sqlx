use std::fmt::{self, Debug, Formatter};

#[cfg(feature = "async")]
use futures_util::future::{BoxFuture, FutureExt};

use super::{MySql, MySqlConnectOptions};
#[cfg(feature = "async")]
use crate::{Async, Result};
use crate::{Close, Connect, Connection, DefaultRuntime, Runtime};
use sqlx_core::Executor;

/// A single connection (also known as a session) to a MySQL database server.
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnection<Rt: Runtime = DefaultRuntime>(
    pub(super) sqlx_mysql::MySqlConnection<Rt>,
);

#[cfg(feature = "async")]
impl<Rt: Async> MySqlConnection<Rt> {
    /// Open a new database connection.
    ///
    /// A value of [`MySqlConnectOptions`] is parsed from the provided
    /// connection `url`.
    ///
    /// ```text
    /// mysql://[[user[:password]@]host][/database][?properties]
    /// ```
    ///
    /// Implemented with [`Connect::connect`][crate::Connect::connect].
    pub async fn connect(url: &str) -> Result<Self> {
        sqlx_mysql::MySqlConnection::<Rt>::connect(url).await.map(Self)
    }

    /// Checks if a connection to the database is still valid.
    ///
    /// Implemented with [`Connection::ping`][crate::Connection::ping].
    pub async fn ping(&mut self) -> Result<()> {
        self.0.ping().await
    }

    pub async fn execute(&mut self, sql: &str) -> Result<()> {
        self.0.execute(sql).await
    }

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as MySQL
    /// will be faster at cleaning up resources.
    ///
    /// Implemented with [`Close::close`][crate::Close::close].
    pub async fn close(self) -> Result<()> {
        self.0.close().await
    }
}

impl<Rt: Runtime> Debug for MySqlConnection<Rt> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    #[inline]
    fn close(self) -> BoxFuture<'static, Result<()>>
    where
        Rt: Async,
    {
        self.close().boxed()
    }
}

impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
    type Options = MySqlConnectOptions<Rt>;

    #[cfg(feature = "async")]
    #[inline]
    fn connect(url: &str) -> BoxFuture<'_, Result<Self>>
    where
        Rt: Async,
    {
        Self::connect(url).boxed()
    }
}

impl<Rt: Runtime> Connection<Rt> for MySqlConnection<Rt> {
    type Database = MySql;

    #[cfg(feature = "async")]
    #[inline]
    fn ping(&mut self) -> BoxFuture<'_, Result<()>>
    where
        Rt: Async,
    {
        self.ping().boxed()
    }
}

impl<Rt: Runtime> Executor<Rt> for MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<()>>
    where
        Rt: Async,
        'e: 'x,
        'q: 'x,
    {
        self.0.execute(sql)
    }
}
