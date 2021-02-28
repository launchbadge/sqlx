use crate::blocking::{Close, Connect, Connection, Executor, Runtime};
use crate::mysql::connection::MySqlConnection;
use crate::mysql::{MySql, MySqlConnectOptions, MySqlQueryResult, MySqlRow};
use crate::{Blocking, Execute, Result, Describe};

impl MySqlConnection<Blocking> {
    /// Open a new database connection.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`connect`](#method.connect).
    ///
    /// Implemented with [`Connect::connect`].
    #[inline]
    pub fn connect(url: &str) -> Result<Self> {
        sqlx_mysql::MySqlConnection::<Blocking>::connect(url).map(Into::into)
    }

    /// Open a new database connection with the configured options.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`connect_with`](#method.connect_with).
    ///
    /// Implemented with [`Connect::connect_with`].
    #[inline]
    pub fn connect_with(options: &MySqlConnectOptions) -> Result<Self> {
        sqlx_mysql::MySqlConnection::<Blocking>::connect_with(options).map(Into::into)
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

impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
    #[inline]
    fn close(self) -> Result<()> {
        self.0.close()
    }
}

impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
    #[inline]
    fn connect_with(options: &MySqlConnectOptions<Rt>) -> Result<Self> {
        sqlx_mysql::MySqlConnection::<Rt>::connect_with(options).map(Into::into)
    }
}

impl<Rt: Runtime> Connection<Rt> for MySqlConnection<Rt> {
    #[inline]
    fn ping(&mut self) -> Result<()> {
        self.0.ping()
    }

    #[inline]
    fn describe<'x, 'e, 'q>(
        &'e mut self,
        query: &'q str,
    ) -> Result<Describe<MySql>>
    where
        'e: 'x,
        'q: 'x,
    {
        self.0.describe(query)
    }
}

impl<Rt: Runtime> Executor<Rt> for MySqlConnection<Rt> {
    #[inline]
    fn execute<'x, 'e, 'q, 'a, E>(&'e mut self, query: E) -> Result<MySqlQueryResult>
    where
        E: 'x + Execute<'q, 'a, MySql>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.0.execute(query)
    }

    #[inline]
    fn fetch_all<'x, 'e, 'q, 'a, E>(&'e mut self, query: E) -> Result<Vec<MySqlRow>>
    where
        E: 'x + Execute<'q, 'a, MySql>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.0.fetch_all(query)
    }

    #[inline]
    fn fetch_optional<'x, 'e, 'q, 'a, E>(&'e mut self, query: E) -> Result<Option<MySqlRow>>
    where
        E: 'x + Execute<'q, 'a, MySql>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.0.fetch_optional(query)
    }
}
