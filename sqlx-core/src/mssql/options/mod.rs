use crate::connection::LogSettings;

mod connect;
mod parse;

#[derive(Debug, Clone)]
pub struct MssqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) database: String,
    pub(crate) password: Option<String>,
    pub(crate) log_settings: LogSettings,
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
            log_settings: Default::default(),
            requested_packet_size: 4096,
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
