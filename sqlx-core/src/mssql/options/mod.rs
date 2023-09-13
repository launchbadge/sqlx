use crate::connection::LogSettings;

mod connect;
mod parse;

/// Options and flags which can be used to configure a Microsoft SQL Server connection.
/// 
/// Connection strings should be in the form:
/// ```text
/// mssql://[username[:password]@]host/database[?instance=instance_name&packet_size=packet_size&client_program_version=client_program_version&client_pid=client_pid&hostname=hostname&app_name=app_name&server_name=server_name&client_interface_name=client_interface_name&language=language]
/// ```
#[derive(Debug, Clone)]
pub struct MssqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) database: String,
    pub(crate) password: Option<String>,
    pub(crate) instance: Option<String>,
    pub(crate) log_settings: LogSettings,
    pub(crate) client_program_version: u32,
    pub(crate) client_pid: u32,
    pub(crate) hostname: String,
    pub(crate) app_name: String,
    pub(crate) server_name: String,
    pub(crate) client_interface_name: String,
    pub(crate) language: String,
    /// Size in bytes of TDS packets to exchange with the server
    pub(crate) requested_packet_size: u32,
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
            instance: None,
            log_settings: Default::default(),
            requested_packet_size: 4096,
            client_program_version: 0,
            client_pid: 0,
            hostname: "".to_string(),
            app_name: "".to_string(),
            server_name: "".to_string(),
            client_interface_name: "".to_string(),
            language: "".to_string(),
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

    pub fn instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_owned());
        self
    }

    pub fn client_program_version(mut self, client_program_version: u32) -> Self {
        self.client_program_version = client_program_version.to_owned();
        self
    }

    pub fn client_pid(mut self, client_pid: u32) -> Self {
        self.client_pid = client_pid.to_owned();
        self
    }

    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = hostname.to_owned();
        self
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = app_name.to_owned();
        self
    }

    pub fn server_name(mut self, server_name: &str) -> Self {
        self.server_name = server_name.to_owned();
        self
    }

    pub fn client_interface_name(mut self, client_interface_name: &str) -> Self {
        self.client_interface_name = client_interface_name.to_owned();
        self
    }

    pub fn language(mut self, language: &str) -> Self {
        self.language = language.to_owned();
        self
    }

    /// Size in bytes of TDS packets to exchange with the server.
    /// Returns an error if the size is smaller than 512 bytes
    pub fn requested_packet_size(mut self, size: u32) -> Result<Self, Self> {
        if size < 512 {
            Err(self)
        } else {
            self.requested_packet_size = size;
            Ok(self)
        }
    }
}
