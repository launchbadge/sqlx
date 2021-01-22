use crate::blocking::{Close, Connect, Connection, Runtime};
use crate::postgres::connection::PostgresConnection;
use crate::{Blocking, Result};

impl PostgresConnection<Blocking> {
    /// Open a new database connection.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`connect`](#method.connect).
    ///
    /// Implemented with [`Connect::connect`].
    #[inline]
    pub fn connect(url: &str) -> Result<Self> {
        sqlx_postgres::PostgresConnection::<Blocking>::connect(url).map(Self)
    }

    /// Checks if a connection to the database is still valid.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`ping`](#method.ping).
    ///
    /// Implemented with [`Connection::ping`].
    #[inline]
    pub fn ping(&mut self) -> Result<()> {
        self.0.ping()
    }

    /// Explicitly close this database connection.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`close`](#method.close).
    ///
    /// Implemented with [`Close::close`].
    #[inline]
    pub fn close(self) -> Result<()> {
        self.0.close()
    }
}

impl<Rt: Runtime> Close<Rt> for PostgresConnection<Rt> {
    #[inline]
    fn close(self) -> Result<()> {
        self.0.close()
    }
}

impl<Rt: Runtime> Connect<Rt> for PostgresConnection<Rt> {
    #[inline]
    fn connect(url: &str) -> Result<Self> {
        sqlx_postgres::PostgresConnection::<Rt>::connect(url).map(Self)
    }
}

impl<Rt: Runtime> Connection<Rt> for PostgresConnection<Rt> {
    #[inline]
    fn ping(&mut self) -> Result<()> {
        self.0.ping()
    }
}
