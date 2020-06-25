use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

use crate::error::{BoxDynError, Error};

/// Options for controlling the desired security state of the connection to the MySQL server.
///
/// It is used by the [`ssl_mode`](MySqlConnectOptions::ssl_mode) method.
#[derive(Debug, Clone, Copy)]
pub enum MySqlSslMode {
    /// Establish an unencrypted connection.
    Disabled,

    /// Establish an encrypted connection if the server supports encrypted connections, falling
    /// back to an unencrypted connection if an encrypted connection cannot be established.
    ///
    /// This is the default if `ssl_mode` is not specified.
    Preferred,

    /// Establish an encrypted connection if the server supports encrypted connections.
    /// The connection attempt fails if an encrypted connection cannot be established.
    Required,

    /// Like `Required`, but additionally verify the server Certificate Authority (CA)
    /// certificate against the configured CA certificates. The connection attempt fails
    /// if no valid matching CA certificates are found.
    VerifyCa,

    /// Like `VerifyCa`, but additionally perform host name identity verification by
    /// checking the host name the client uses for connecting to the server against the
    /// identity in the certificate that the server sends to the client.
    VerifyIdentity,
}

impl Default for MySqlSslMode {
    fn default() -> Self {
        MySqlSslMode::Preferred
    }
}

impl FromStr for MySqlSslMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            "DISABLED" => MySqlSslMode::Disabled,
            "PREFERRED" => MySqlSslMode::Preferred,
            "REQUIRED" => MySqlSslMode::Required,
            "VERIFY_CA" => MySqlSslMode::VerifyCa,
            "VERIFY_IDENTITY" => MySqlSslMode::VerifyIdentity,

            _ => {
                return Err(err_protocol!("unknown SSL mode value: {:?}", s));
            }
        })
    }
}

/// Options and flags which can be used to configure a MySQL connection.
///
/// A value of `PgConnectOptions` can be parsed from a connection URI,
/// as described by [MySQL](https://dev.mysql.com/doc/connector-j/8.0/en/connector-j-reference-jdbc-url-format.html).
///
/// The generic format of the connection URL:
///
/// ```text
/// mysql://[host][/database][?properties]
/// ```
///
/// # Example
///
/// ```rust,no_run
/// # use sqlx_core::error::Error;
/// # use sqlx_core::connection::Connect;
/// # use sqlx_core::mysql::{MySqlConnectOptions, MySqlConnection, MySqlSslMode};
/// #
/// # fn main() -> Result<(), Error> {
/// # #[cfg(feature = "runtime-async-std")]
/// # sqlx_rt::async_std::task::block_on(async move {
/// // URI connection string
/// let conn = MySqlConnection::connect("mysql://root:password@localhost/db").await?;
///
/// // Manually-constructed options
/// let conn = MySqlConnection::connect_with(&MySqlConnectOptions::new()
///     .host("localhost")
///     .username("root")
///     .password("password")
///     .database("db")
/// ).await?;
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MySqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) ssl_mode: MySqlSslMode,
    pub(crate) ssl_ca: Option<PathBuf>,
}

impl Default for MySqlConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlConnectOptions {
    /// Creates a new, default set of options ready for configuration
    pub fn new() -> Self {
        Self {
            port: 3306,
            host: String::from("localhost"),
            username: String::from("root"),
            password: None,
            database: None,
            ssl_mode: MySqlSslMode::Preferred,
            ssl_ca: None,
        }
    }

    /// Sets the name of the host to connect to.
    ///
    /// The default behavior when the host is not specified,
    /// is to connect to localhost.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_owned();
        self
    }

    /// Sets the port to connect to at the server host.
    ///
    /// The default port for MySQL is `3306`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the username to connect as.
    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_owned();
        self
    }

    /// Sets the password to connect with.
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    /// Sets the database name.
    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_owned());
        self
    }

    /// Sets whether or with what priority a secure SSL TCP/IP connection will be negotiated
    /// with the server.
    ///
    /// By default, the SSL mode is [`Preferred`](MySqlSslMode::Preferred), and the client will
    /// first attempt an SSL connection but fallback to a non-SSL connection on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::mysql::{MySqlSslMode, MySqlConnectOptions};
    /// let options = MySqlConnectOptions::new()
    ///     .ssl_mode(MySqlSslMode::Required);
    /// ```
    pub fn ssl_mode(mut self, mode: MySqlSslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// Sets the name of a file containing a list of trusted SSL Certificate Authorities.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::mysql::{MySqlSslMode, MySqlConnectOptions};
    /// let options = MySqlConnectOptions::new()
    ///     .ssl_mode(MySqlSslMode::VerifyCa)
    ///     .ssl_ca("path/to/ca.crt");
    /// ```
    pub fn ssl_ca(mut self, file_name: impl AsRef<Path>) -> Self {
        self.ssl_ca = Some(file_name.as_ref().to_owned());
        self
    }
}

impl FromStr for MySqlConnectOptions {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, BoxDynError> {
        let url: Url = s.parse()?;
        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options = options.host(host);
        }

        if let Some(port) = url.port() {
            options = options.port(port);
        }

        let username = url.username();
        if !username.is_empty() {
            options = options.username(username);
        }

        if let Some(password) = url.password() {
            options = options.password(password);
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(path);
        }

        for (key, value) in url.query_pairs().into_iter() {
            match &*key {
                "ssl-mode" => {
                    options = options.ssl_mode(value.parse()?);
                }

                "ssl-ca" => {
                    options = options.ssl_ca(&*value);
                }

                _ => {}
            }
        }

        Ok(options)
    }
}
