use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use either::Either;
use sqlx_core::{ConnectOptions, DefaultRuntime, Runtime};

use crate::MySqlConnection;

mod builder;
mod default;
mod parse;

// TODO: RSA Public Key (to avoid the key exchange for caching_sha2 and sha256 plugins)

/// Options which can be used to configure how a MySQL connection is opened.
///
/// A value of `MySqlConnectOptions` can be parsed from a connection URL,
/// as described by the [MySQL JDBC connector reference](https://dev.mysql.com/doc/connector-j/8.0/en/connector-j-reference-jdbc-url-format.html).
///
/// ```text
/// mysql://[host][/database][?properties]
/// ```
///
///  - The protocol must be `mysql`.
///
///  - Only a single host is supported.
///
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnectOptions<Rt = DefaultRuntime>
where
    Rt: Runtime,
{
    runtime: PhantomData<Rt>,
    pub(crate) address: Either<(String, u16), PathBuf>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    timezone: String,
    charset: String,
}

impl<Rt> Clone for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn clone(&self) -> Self {
        Self {
            runtime: PhantomData,
            address: self.address.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            database: self.database.clone(),
            timezone: self.timezone.clone(),
            charset: self.charset.clone(),
        }
    }
}

impl<Rt> Debug for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnectOptions")
            .field(
                "address",
                &self
                    .address
                    .as_ref()
                    .map_left(|(host, port)| format!("{}:{}", host, port))
                    .map_right(|socket| socket.display()),
            )
            .field("username", &self.username)
            .field("password", &self.password)
            .field("database", &self.database)
            .field("timezone", &self.timezone)
            .field("charset", &self.charset)
            .finish()
    }
}

impl<Rt> MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    /// Returns the hostname of the database server.
    #[must_use]
    pub fn get_host(&self) -> &str {
        self.address.as_ref().left().map_or(default::HOST, |(host, _)| &**host)
    }

    /// Returns the TCP port number of the database server.
    #[must_use]
    pub fn get_port(&self) -> u16 {
        self.address.as_ref().left().map_or(default::PORT, |(_, port)| *port)
    }

    /// Returns the path to the Unix domain socket, if one is configured.
    #[must_use]
    pub fn get_socket(&self) -> Option<&Path> {
        self.address.as_ref().right().map(PathBuf::as_path)
    }

    /// Returns the default database name.
    #[must_use]
    pub fn get_database(&self) -> Option<&str> {
        self.database.as_deref()
    }

    /// Returns the username to be used for authentication.
    #[must_use]
    pub fn get_username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns the password to be used for authentication.
    #[must_use]
    pub fn get_password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Returns the character set for the connection.
    #[must_use]
    pub fn get_charset(&self) -> &str {
        &self.charset
    }

    /// Returns the timezone for the connection.
    #[must_use]
    pub fn get_timezone(&self) -> &str {
        &self.timezone
    }
}

impl<Rt> ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    type Connection = MySqlConnection<Rt>;

    #[cfg(feature = "async")]
    fn connect(&self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<Self::Connection>>
    where
        Self::Connection: Sized,
        Rt: sqlx_core::Async,
    {
        Box::pin(MySqlConnection::<Rt>::connect_async(self))
    }
}

#[cfg(feature = "blocking")]
impl<Rt> sqlx_core::blocking::ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: sqlx_core::blocking::Runtime,
{
    fn connect(&self) -> sqlx_core::Result<Self::Connection>
    where
        Self::Connection: Sized,
    {
        <MySqlConnection<Rt>>::connect(self)
    }
}
