use crate::{connection::LogSettings, net::CertificateInput};
use std::path::Path;

mod connect;
mod parse;
mod ssl_mode;

pub use ssl_mode::MssqlSslMode;

#[derive(Debug, Clone)]
pub struct MssqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) database: String,
    pub(crate) password: Option<String>,
    pub(crate) log_settings: LogSettings,
    pub(crate) ssl_mode: MssqlSslMode,
    pub(crate) ssl_ca: Option<CertificateInput>,
}

impl Default for MssqlConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MssqlConnectOptions {
    pub fn new() -> Self {
        Self {
            port: 1433,
            host: String::from("localhost"),
            database: String::from("master"),
            username: String::from("sa"),
            password: None,
            log_settings: Default::default(),
            ssl_mode: MssqlSslMode::Preferred,
            ssl_ca: None,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_owned();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_owned();
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    pub fn database(mut self, database: &str) -> Self {
        self.database = database.to_owned();
        self
    }

    /// Sets whether or with what priority a secure SSL TCP/IP connection will be negotiated
    /// with the server.
    ///
    /// By default, the SSL mode is [`Preferred`](MssqlSslMode::Preferred), and the client will
    /// first attempt an SSL connection but fallback to a non-SSL connection on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::mysql::{MssqlSslMode, MySqlConnectOptions};
    /// let options = MssqlConnectOptions::new()
    ///     .ssl_mode(MssqlSslMode::Required);
    /// ```
    pub fn ssl_mode(mut self, mode: MssqlSslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// Sets the name of a file containing a list of trusted SSL Certificate Authorities.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::mssql::{MssqlSslMode, MssqlConnectOptions};
    /// let options = MssqlConnectOptions::new()
    ///     .ssl_mode(MssqlSslMode::Preferred)
    ///     .ssl_ca("path/to/ca.crt");
    /// ```
    pub fn ssl_ca(mut self, file_name: impl AsRef<Path>) -> Self {
        self.ssl_ca = Some(CertificateInput::File(file_name.as_ref().to_owned()));
        self
    }

    /// Sets PEM encoded list of trusted SSL Certificate Authorities.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_core::mssql::{MssqlSslMode, MssqlConnectOptions};
    /// let options = MssqlConnectOptions::new()
    ///     .ssl_mode(MssqlSslMode::Preferred)
    ///     .ssl_ca_from_pem(vec![]);
    /// ```
    pub fn ssl_ca_from_pem(mut self, pem_certificate: Vec<u8>) -> Self {
        self.ssl_ca = Some(CertificateInput::Inline(pem_certificate));
        self
    }
}
