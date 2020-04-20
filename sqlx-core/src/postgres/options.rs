use std::convert::TryFrom;
use std::env::var;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::{BoxDynError, Error};
use crate::io::BufStream;
use crate::net::Socket;
use crate::postgres::connection::PgConnection;
use url::Url;

/// Options for controlling the level of protection provided for PostgreSQL SSL connections.
///
/// It is used by the [`ssl_mode`](PgConnectOptions::ssl_mode) method.
#[derive(Debug, Clone, Copy)]
pub enum PgSslMode {
    /// Only try a non-SSL connection.
    Disable,

    /// First try a non-SSL connection; if that fails, try an SSL connection.
    Allow,

    /// First try an SSL connection; if that fails, try a non-SSL connection.
    Prefer,

    /// Only try an SSL connection. If a root CA file is present, verify the connection
    /// in the same way as if `VerifyCa` was specified.
    Require,

    /// Only try an SSL connection, and verify that the server certificate is issued by a
    /// trusted certificate authority (CA).
    VerifyCa,

    /// Only try an SSL connection; verify that the server certificate is issued by a trusted
    /// CA and that the requested server host name matches that in the certificate.
    VerifyFull,
}

impl Default for PgSslMode {
    fn default() -> Self {
        PgSslMode::Prefer
    }
}

impl FromStr for PgSslMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            "disable" => PgSslMode::Disable,
            "allow" => PgSslMode::Allow,
            "prefer" => PgSslMode::Prefer,
            "require" => PgSslMode::Require,
            "verify-ca" => PgSslMode::VerifyCa,
            "verify-full" => PgSslMode::VerifyFull,

            _ => {
                return Err(err_protocol!("unknown SSL mode value: {:?}", s));
            }
        })
    }
}

/// Options and flags which can be used to configure a PostgreSQL connection.
///
/// A value of `PgConnectOptions` can be parsed from a connection string, in one of the
/// forms supported by [libpq](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING).
///
/// In the first format, each parameter is in the format `keyword = value`. Spaces around the `=`
/// are optional. To write an empty value, or a value containing spaces, surround it with single
/// quotes. Single quotes and backslashes within the value must be escaped with a backslash, i.e., `\'` and `\\`.
///
/// ```text
/// host=localhost port=5432 dbname=mydb
/// ```
///
/// The general form for a connection URI is:
///
/// ```text
/// postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
/// ```
///
/// The URI scheme designator can be either `postgresql://` or `postgres://`.
/// Each of the URI parts is optional.
///
/// ```text
/// postgresql://
/// postgresql://localhost
/// postgresql://localhost:5433
/// postgresql://localhost/mydb
/// postgresql://user@localhost
/// postgresql://user:secret@localhost
/// ```
///
/// # Example
///
/// ```rust,no_run
/// # use sqlx_core::error::Error;
/// # use sqlx_core::connection::Connect;
/// # use sqlx_core::postgres::{PgConnectOptions, PgConnection, PgSslMode};
/// #
/// # #[cfg(feature = "runtime-async-std")]
/// # #[async_std::main]
/// # async fn main() -> Result<(), Error> {
/// // Keyword/Value connection string
/// // TODO: let conn = PgConnection::connect("host=localhost port=5432 dbname=mydb").await?;
///
/// // URI connection string
/// // TODO: let conn = PgConnection::connect("postgres://localhost/mydb").await?;
///
/// // Manually-constructed options
/// let conn = PgConnection::connect(PgConnectOptions::new()
///     .host("secret-host")
///     .port(2525)
///     .username("secret-user")
///     .password("secret-password")
///     .ssl_mode(PgSslMode::Require)).await?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "runtime-async-std"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone)]
pub struct PgConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) ssl_mode: PgSslMode,
    pub(crate) ssl_root_cert: Option<PathBuf>,
}

impl PgConnectOptions {
    /// Creates a new, default set of options ready for configuration.
    ///
    /// By default, this reads the following environment variables and sets their
    /// equivalent options.
    ///
    ///  * `PGHOST`
    ///  * `PGPORT`
    ///  * `PGUSER`
    ///  * `PGPASSWORD`
    ///  * `PGDATABASE`
    ///  * `PGSSLROOTCERT`
    ///  * `PGSSLMODE`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::PgConnectOptions;
    /// let options = PgConnectOptions::new();
    /// ```
    pub fn new() -> Self {
        let port = var("PGPORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5432);

        let host = var("PGHOST").ok().unwrap_or_else(|| default_host(port));

        PgConnectOptions {
            port,
            host,
            username: var("PGUSER").ok().unwrap_or_else(whoami::username),
            password: var("PGPASSWORD").ok(),
            database: var("PGDATABASE").ok(),
            ssl_root_cert: var("PGSSLROOTCERT").ok().map(PathBuf::from),
            ssl_mode: var("PGSSLMODE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_default(),
        }
    }

    /// Sets the name of the host to connect to.
    ///
    /// If a host name begins with a slash, it specifies
    /// Unix-domain communication rather than TCP/IP communication; the value is the name of
    /// the directory in which the socket file is stored.
    ///
    /// The default behavior when host is not specified, or is empty,
    /// is to connect to a Unix-domain socket
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::PgConnectOptions;
    /// let options = PgConnectOptions::new()
    ///     .host("localhost");
    /// ```
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_owned();
        self
    }

    /// Sets the port connect to at the server host.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::PgConnectOptions;
    /// let options = PgConnectOptions::new()
    ///     .port(5432);
    /// ```
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the username to connect as.
    ///
    /// Defaults to be the same as the operating system name of
    /// the user running the application.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::PgConnectOptions;
    /// let options = PgConnectOptions::new()
    ///     .username("postgres");
    /// ```
    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_owned();
        self
    }

    /// Sets the password to use if the server demands password authentication.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::PgConnectOptions;
    /// let options = PgConnectOptions::new()
    ///     .username("root")
    ///     .password("safe-and-secure");
    /// ```
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    /// Sets the database name. Defaults to be the same as the user name.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::PgConnectOptions;
    /// let options = PgConnectOptions::new()
    ///     .database("postgres");
    /// ```
    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_owned());
        self
    }

    /// Sets whether or with what priority a secure SSL TCP/IP connection will be negotiated
    /// with the server.
    ///
    /// By default, the SSL mode is [`Prefer`](PgSslMode::Prefer), and the client will
    /// first attempt an SSL connection but fallback to a non-SSL connection on failure.
    ///
    /// Ignored for Unix domain socket communication.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::{PgSslMode, PgConnectOptions};
    /// let options = PgConnectOptions::new()
    ///     .ssl_mode(PgSslMode::Require);
    /// ```
    pub fn ssl_mode(mut self, mode: PgSslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// Sets the name of a file containing SSL certificate authority (CA) certificate(s).
    /// If the file exists, the server's certificate will be verified to be signed by
    /// one of these authorities.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::postgres::{PgSslMode, PgConnectOptions};
    /// let options = PgConnectOptions::new()
    ///     // Providing a CA certificate with less than VerifyCa is pointless
    ///     .ssl_mode(PgSslMode::VerifyCa)
    ///     .ssl_root_cert("./ca-certificate.crt");
    /// ```
    pub fn ssl_root_cert(mut self, cert: impl AsRef<Path>) -> Self {
        self.ssl_root_cert = Some(cert.as_ref().to_path_buf());
        self
    }
}

fn default_host(port: u16) -> String {
    // try to check for the existence of a unix socket and uses that
    let socket = format!(".s.PGSQL.{}", port);
    let candidates = [
        "/var/run/postgresql", // Debian
        "/private/tmp",        // OSX (homebrew)
        "/tmp",                // Default
    ];

    'outer: loop {
        for candidate in &candidates {
            if Path::new(candidate).join(&socket).exists() {
                break 'outer candidate.to_string();
            }
        }

        // fallback to localhost if no socket was found
        break "localhost".to_owned();
    }
}

impl FromStr for PgConnectOptions {
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
                "sslmode" => {
                    options = options.ssl_mode(value.parse()?);
                }

                "sslrootcert" => {
                    options = options.ssl_root_cert(&*value);
                }

                // TODO: Should we error on unrecognized keys?
                _ => {}
            }
        }

        Ok(options)
    }
}
