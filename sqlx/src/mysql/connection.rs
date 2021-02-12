use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "async")]
use futures_util::future::{BoxFuture, FutureExt};
use sqlx_core::Executor;

use super::{MySql, MySqlConnectOptions, MySqlQueryResult, MySqlRow};
#[cfg(feature = "blocking")]
use crate::blocking;
#[cfg(feature = "async")]
use crate::{Async, Result};
use crate::{Close, Connect, Connection, DefaultRuntime, Runtime};

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

    /// Open a new database connection with the configured options.
    ///
    /// Implemented with [`Connect::connect_with`][crate::Connect::connect_with].
    pub async fn connect_with(options: &MySqlConnectOptions<Rt>) -> Result<Self> {
        sqlx_mysql::MySqlConnection::<Rt>::connect_with(&**options).await.map(Self)
    }

    /// Checks if a connection to the database is still valid.
    ///
    /// Implemented with [`Connection::ping`][crate::Connection::ping].
    pub async fn ping(&mut self) -> Result<()> {
        self.0.ping().await
    }

    // TODO: document from Executor

    pub async fn execute(&mut self, sql: &str) -> Result<MySqlQueryResult> {
        self.0.execute(sql).await
    }

    pub async fn fetch_all(&mut self, sql: &str) -> Result<Vec<MySqlRow>> {
        self.0.fetch_all(sql).await
    }

    pub async fn fetch_one(&mut self, sql: &str) -> Result<MySqlRow> {
        self.0.fetch_one(sql).await
    }

    pub async fn fetch_optional(&mut self, sql: &str) -> Result<Option<MySqlRow>> {
        self.0.fetch_optional(sql).await
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
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self>>
    where
        Rt: Async,
    {
        Self::connect_with(options).boxed()
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
        self.0.ping()
    }
}

impl<Rt: Runtime> Executor<Rt> for MySqlConnection<Rt> {
    type Database = MySql;

    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<MySqlQueryResult>>
    where
        Rt: Async,
        'e: 'x,
        'q: 'x,
    {
        self.0.execute(sql)
    }

    fn fetch_all<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<Vec<MySqlRow>>>
    where
        Rt: Async,
        'e: 'x,
        'q: 'x,
    {
        self.0.fetch_all(sql)
    }

    fn fetch_optional<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> BoxFuture<'x, Result<Option<MySqlRow>>>
    where
        Rt: Async,
        'e: 'x,
        'q: 'x,
    {
        self.0.fetch_optional(sql)
    }
}

impl<Rt: Runtime> From<sqlx_mysql::MySqlConnection<Rt>> for MySqlConnection<Rt> {
    fn from(connection: sqlx_mysql::MySqlConnection<Rt>) -> Self {
        Self(connection)
    }
}

impl<Rt: Runtime> Deref for MySqlConnection<Rt> {
    type Target = sqlx_mysql::MySqlConnection<Rt>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Rt: Runtime> DerefMut for MySqlConnection<Rt> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
