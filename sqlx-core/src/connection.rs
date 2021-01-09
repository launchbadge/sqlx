#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

use crate::{ConnectOptions, Database, DefaultRuntime, Runtime};

/// A unique connection (session) with a specific database.
///
/// With a client/server model, this is equivalent to a network connection
/// to the server.
///
/// SQL statements will be executed and results returned within the context
/// of this single SQL connection.
///
pub trait Connection<Rt = DefaultRuntime>: 'static + Send
where
    Rt: Runtime,
{
    type Database: Database<Rt, Connection = Self>;

    type Options: ConnectOptions<Rt, Connection = Self>;

    /// Establish a new database connection.
    ///
    /// A value of [`Options`](#associatedtype.Options) is parsed from the provided connection string. This parsing
    /// is database-specific.
    ///
    /// ```rust,ignore
    /// use sqlx::postgres::PgConnection;
    ///
    /// let mut conn = <PgConnection>::connect(
    ///     "postgres://postgres:password@localhost/database",
    /// ).await?;
    /// ```
    ///
    /// You may alternatively build the connection options imperatively.
    ///
    /// ```rust,ignore
    /// use sqlx::mysql::MySqlConnectOptions;
    /// use sqlx::ConnectOptions;
    ///
    /// let mut conn = <MySqlConnectOptions>::new()
    ///     .host("localhost")
    ///     .username("root")
    ///     .password("password")
    ///     .connect().await?;
    /// ```
    ///
    #[cfg(feature = "async")]
    #[must_use]
    fn connect(url: &str) -> BoxFuture<'_, crate::Result<Self>>
    where
        Self: Sized,
        Rt: crate::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        let options = url.parse::<Self::Options>();
        Box::pin(async move { options?.connect().await })
    }

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database backend
    /// will be faster at cleaning up resources.
    ///
    #[cfg(feature = "async")]
    fn close(self) -> BoxFuture<'static, crate::Result<()>>
    where
        Rt: crate::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin;

    /// Checks if a connection to the database is still valid.
    ///
    /// The method of operation greatly depends on the database driver. In MySQL, there is an
    /// explicit [`COM_PING`](https://dev.mysql.com/doc/internals/en/com-ping.html) command. In
    /// PostgreSQL, `ping` will issue a query consisting of a comment `/* SQLx ping */` which,
    /// in effect, does nothing apart from getting a response from the server.
    ///
    #[cfg(feature = "async")]
    fn ping(&mut self) -> BoxFuture<'_, crate::Result<()>>
    where
        Rt: crate::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin;
}
