mod connect;
mod parse;
pub mod ssl_mode;

use crate::connection::LogSettings;
use ssl_mode::MssqlSslMode;

/// Options and flags which can be used to configure a MSSQL connection.
///
/// A value of `MssqlConnectOptions` can be parsed from a connection URL,
/// as described below.
///
/// The generic format of the connection URL:
///
/// ```text
/// mssql://[user[:password]@]host[:port][/database][?properties]
/// ```
///
/// ## Properties
///
/// |Parameter|Default|Description|
/// |---------|-------|-----------|
/// | `sslmode` / `ssl_mode` | `preferred` | SSL encryption mode: `disabled`, `login_only`, `preferred`, `required`. |
/// | `encrypt` | (none) | Legacy alias: `true` maps to `required`, `false` to `disabled`. |
/// | `trust_server_certificate` | `false` | Whether to trust the server certificate without validation. |
/// | `trust_server_certificate_ca` | (none) | Path to a CA certificate file to validate the server certificate against. Mutually exclusive with `trust_server_certificate`. |
/// | `application_intent` | `read_write` | Application intent: `read_write` or `read_only`. `read_only` routes to Always On read replicas. |
/// | `statement-cache-capacity` | `100` | The maximum number of prepared statements stored in the cache. |
/// | `app_name` | `sqlx` | The application name sent to the server. |
/// | `instance` | `None` | The SQL Server instance name. |
/// | `auth` | `sql_server` | Authentication method: `sql_server`, `windows` (cfg-gated), `integrated` (cfg-gated), `aad_token`. |
/// | `token` | (none) | Azure AD bearer token (used when `auth=aad_token`). |
///
/// # Example
///
/// ```rust,no_run
/// # async fn example() -> sqlx::Result<()> {
/// use sqlx::{Connection, ConnectOptions};
/// use sqlx::mssql::{MssqlConnectOptions, MssqlConnection};
///
/// // URL connection string
/// let conn = MssqlConnection::connect("mssql://sa:password@localhost/master").await?;
///
/// // Manually-constructed options
/// let conn = MssqlConnectOptions::new()
///     .host("localhost")
///     .username("sa")
///     .password("password")
///     .database("master")
///     .connect().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MssqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) instance: Option<String>,
    pub(crate) ssl_mode: MssqlSslMode,
    pub(crate) trust_server_certificate: bool,
    pub(crate) trust_server_certificate_ca: Option<String>,
    pub(crate) application_intent_read_only: bool,
    pub(crate) statement_cache_capacity: usize,
    pub(crate) app_name: String,
    pub(crate) log_settings: LogSettings,
    /// When `true`, use Windows (NTLM) authentication instead of SQL Server auth.
    /// The username can use `domain\user` syntax which tiberius parses internally.
    #[cfg(all(windows, feature = "winauth"))]
    pub(crate) windows_auth: bool,
    /// When `true`, use integrated authentication (SSPI on Windows / Kerberos on Unix).
    #[cfg(any(
        all(windows, feature = "winauth"),
        all(unix, feature = "integrated-auth-gssapi")
    ))]
    pub(crate) integrated_auth: bool,
    /// Azure AD bearer token for AAD authentication.
    pub(crate) aad_token: Option<String>,
}

impl Default for MssqlConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MssqlConnectOptions {
    /// Creates a new, default set of options ready for configuration.
    pub fn new() -> Self {
        Self {
            port: 1433,
            host: String::from("localhost"),
            username: String::from("sa"),
            password: None,
            database: None,
            instance: None,
            ssl_mode: MssqlSslMode::default(),
            trust_server_certificate: false,
            trust_server_certificate_ca: None,
            application_intent_read_only: false,
            statement_cache_capacity: 100,
            app_name: String::from("sqlx"),
            log_settings: Default::default(),
            #[cfg(all(windows, feature = "winauth"))]
            windows_auth: false,
            #[cfg(any(
                all(windows, feature = "winauth"),
                all(unix, feature = "integrated-auth-gssapi")
            ))]
            integrated_auth: false,
            aad_token: None,
        }
    }

    /// Sets the name of the host to connect to.
    pub fn host(mut self, host: &str) -> Self {
        host.clone_into(&mut self.host);
        self
    }

    /// Sets the port to connect to at the server host.
    ///
    /// The default port for MSSQL is `1433`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the username to connect as.
    pub fn username(mut self, username: &str) -> Self {
        username.clone_into(&mut self.username);
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

    /// Sets the SQL Server instance name.
    pub fn instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_owned());
        self
    }

    /// Sets the SSL encryption mode.
    pub fn ssl_mode(mut self, mode: MssqlSslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// Sets whether to use TLS encryption.
    ///
    /// This is a legacy convenience method.
    /// `true` maps to [`MssqlSslMode::Required`], `false` to [`MssqlSslMode::Disabled`].
    pub fn encrypt(mut self, encrypt: bool) -> Self {
        self.ssl_mode = if encrypt {
            MssqlSslMode::Required
        } else {
            MssqlSslMode::Disabled
        };
        self
    }

    /// Sets whether to trust the server certificate without validation.
    pub fn trust_server_certificate(mut self, trust: bool) -> Self {
        self.trust_server_certificate = trust;
        self
    }

    /// Sets a CA certificate file path to validate the server certificate against.
    ///
    /// Accepts `.pem`, `.crt`, or `.der` certificate files.
    ///
    /// This is mutually exclusive with [`trust_server_certificate`](Self::trust_server_certificate).
    /// When a CA path is set, `trust_server_certificate` is ignored.
    pub fn trust_server_certificate_ca(mut self, path: &str) -> Self {
        self.trust_server_certificate_ca = Some(path.to_owned());
        self
    }

    /// Sets the application intent to read-only.
    ///
    /// When `true`, sets `ApplicationIntent=ReadOnly` in the TDS login packet,
    /// which routes connections to Always On Availability Group read replicas.
    pub fn application_intent_read_only(mut self, read_only: bool) -> Self {
        self.application_intent_read_only = read_only;
        self
    }

    /// Get whether the application intent is set to read-only.
    pub fn get_application_intent_read_only(&self) -> bool {
        self.application_intent_read_only
    }

    /// Sets the capacity of the connection's statement cache.
    pub fn statement_cache_capacity(mut self, capacity: usize) -> Self {
        self.statement_cache_capacity = capacity;
        self
    }

    /// Sets the application name sent to the server.
    pub fn app_name(mut self, app_name: &str) -> Self {
        app_name.clone_into(&mut self.app_name);
        self
    }

    /// Sets whether to use Windows (NTLM) authentication.
    ///
    /// When enabled, the username can use `domain\user` syntax
    /// which tiberius parses internally.
    #[cfg(all(windows, feature = "winauth"))]
    pub fn windows_auth(mut self, enabled: bool) -> Self {
        self.windows_auth = enabled;
        self
    }

    /// Sets whether to use integrated authentication (SSPI on Windows / Kerberos on Unix).
    #[cfg(any(
        all(windows, feature = "winauth"),
        all(unix, feature = "integrated-auth-gssapi")
    ))]
    pub fn integrated_auth(mut self, enabled: bool) -> Self {
        self.integrated_auth = enabled;
        self
    }

    /// Sets an Azure AD bearer token for authentication.
    ///
    /// When set, AAD token authentication takes precedence over other auth methods.
    pub fn aad_token(mut self, token: &str) -> Self {
        self.aad_token = Some(token.to_owned());
        self
    }

    /// Get the current host.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get the server's port.
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Get the current username.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Get the current database name.
    pub fn get_database(&self) -> Option<&str> {
        self.database.as_deref()
    }

    /// Build a `tiberius::Config` from these options.
    pub(crate) fn to_tiberius_config(&self) -> tiberius::Config {
        let mut config = tiberius::Config::new();

        config.host(&self.host);
        config.port(self.port);
        config.application_name(&self.app_name);

        if let Some(database) = &self.database {
            config.database(database);
        }

        if let Some(instance) = &self.instance {
            config.instance_name(instance);
        }

        if let Some(token) = &self.aad_token {
            config.authentication(tiberius::AuthMethod::aad_token(token));
        } else {
            #[allow(unused_mut)]
            let mut handled = false;

            #[cfg(any(
                all(windows, feature = "winauth"),
                all(unix, feature = "integrated-auth-gssapi")
            ))]
            if !handled && self.integrated_auth {
                config.authentication(tiberius::AuthMethod::Integrated);
                handled = true;
            }

            #[cfg(all(windows, feature = "winauth"))]
            if !handled && self.windows_auth {
                config.authentication(tiberius::AuthMethod::windows(
                    &self.username,
                    self.password.as_deref().unwrap_or(""),
                ));
                handled = true;
            }

            if !handled {
                config.authentication(tiberius::AuthMethod::sql_server(
                    &self.username,
                    self.password.as_deref().unwrap_or(""),
                ));
            }
        }

        if let Some(ca_path) = &self.trust_server_certificate_ca {
            // trust_cert_ca and trust_cert are mutually exclusive in tiberius
            config.trust_cert_ca(ca_path);
        } else if self.trust_server_certificate {
            config.trust_cert();
        }

        if self.application_intent_read_only {
            config.readonly(true);
        }

        config.encryption(match self.ssl_mode {
            MssqlSslMode::Disabled => tiberius::EncryptionLevel::NotSupported,
            MssqlSslMode::LoginOnly => tiberius::EncryptionLevel::Off,
            MssqlSslMode::Preferred => tiberius::EncryptionLevel::On,
            MssqlSslMode::Required => tiberius::EncryptionLevel::Required,
        });

        config
    }
}
