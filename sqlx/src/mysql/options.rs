use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(feature = "async")]
use futures_util::future::{BoxFuture, FutureExt};

use super::MySqlConnection;
#[cfg(feature = "async")]
use crate::Async;
use crate::{ConnectOptions, DefaultRuntime, Error, Result, Runtime};

/// Options which can be used to configure how a MySQL connection is opened.
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnectOptions<Rt: Runtime = DefaultRuntime> {
    runtime: PhantomData<Rt>,
    options: sqlx_mysql::MySqlConnectOptions,
}

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
        Ok(url.parse::<sqlx_mysql::MySqlConnectOptions>()?.into())
    }
}

#[cfg(feature = "async")]
impl<Rt: Async> MySqlConnectOptions<Rt> {
    /// Open a new database connection with the configured connection options.
    ///
    /// Implemented with [`ConnectOptions::connect`].
    #[inline]
    pub async fn connect(&self) -> Result<MySqlConnection<Rt>> {
        <Self as ConnectOptions>::connect::<MySqlConnection<Rt>, Rt>(self).await
    }
}

// explicitly forwards builder methods
// in order to return Self as sqlx::mysql::MySqlConnectOptions instead of
// sqlx_mysql::MySqlConnectOptions
impl<Rt: Runtime> MySqlConnectOptions<Rt> {
    /// Sets the hostname of the database server.
    ///
    /// If the hostname begins with a slash (`/`), it is interpreted as the absolute path
    /// to a Unix domain socket file instead of a hostname of a server.
    ///
    /// Defaults to `localhost`.
    ///
    pub fn host(&mut self, host: impl AsRef<str>) -> &mut Self {
        self.options.host(host);
        self
    }

    /// Sets the path of the Unix domain socket to connect to.
    ///
    /// Overrides [`host()`](#method.host) and [`port()`](#method.port).
    ///
    pub fn socket(&mut self, socket: impl AsRef<Path>) -> &mut Self {
        self.options.socket(socket);
        self
    }

    /// Sets the TCP port number of the database server.
    ///
    /// Defaults to `3306`.
    ///
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.options.port(port);
        self
    }

    /// Sets the username to be used for authentication.
    // FIXME: Specify what happens when you do NOT set this
    pub fn username(&mut self, username: impl AsRef<str>) -> &mut Self {
        self.options.username(username);
        self
    }

    /// Sets the password to be used for authentication.
    pub fn password(&mut self, password: impl AsRef<str>) -> &mut Self {
        self.options.password(password);
        self
    }

    /// Sets the default database for the connection.
    pub fn database(&mut self, database: impl AsRef<str>) -> &mut Self {
        self.options.database(database);
        self
    }

    /// Sets the character set for the connection.
    pub fn charset(&mut self, charset: impl AsRef<str>) -> &mut Self {
        self.options.charset(charset);
        self
    }

    /// Sets the timezone for the connection.
    pub fn timezone(&mut self, timezone: impl AsRef<str>) -> &mut Self {
        self.options.timezone(timezone);
        self
    }
}

// allow trivial conversion from [sqlx_mysql::MySqlConnectOptions] to
// our runtime-wrapped [sqlx::mysql::MySqlConnectOptions]
impl<Rt: Runtime> From<sqlx_mysql::MySqlConnectOptions> for MySqlConnectOptions<Rt> {
    #[inline]
    fn from(options: sqlx_mysql::MySqlConnectOptions) -> Self {
        Self { runtime: PhantomData, options }
    }
}

// default implement [ConnectOptions]
// ensures that the required traits for [MySqlConnectOptions<Rt>] are implemented
impl<Rt: Runtime> ConnectOptions for MySqlConnectOptions<Rt> {}

// forward Debug to [sqlx_mysql::MySqlConnectOptions]
impl<Rt: Runtime> Debug for MySqlConnectOptions<Rt> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.options)
    }
}

// forward Default to [sqlx_mysql::MySqlConnectOptions]
impl<Rt: Runtime> Default for MySqlConnectOptions<Rt> {
    fn default() -> Self {
        sqlx_mysql::MySqlConnectOptions::default().into()
    }
}

// forward Clone to [sqlx_mysql::MySqlConnectOptions]
impl<Rt: Runtime> Clone for MySqlConnectOptions<Rt> {
    fn clone(&self) -> Self {
        Self { runtime: PhantomData, options: self.options.clone() }
    }
}

// forward FromStr to [sqlx_mysql::MySqlConnectOptions]
impl<Rt: Runtime> FromStr for MySqlConnectOptions<Rt> {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self> {
        Self::parse(url)
    }
}

// allow dereferencing into [sqlx_mysql::MySqlConnectOptions]
// note that we do not allow mutable dereferencing as those methods have the wrong return type
impl<Rt: Runtime> Deref for MySqlConnectOptions<Rt> {
    type Target = sqlx_mysql::MySqlConnectOptions;

    fn deref(&self) -> &Self::Target {
        &self.options
    }
}
