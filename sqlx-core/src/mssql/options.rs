use std::str::FromStr;

use url::Url;

use crate::error::BoxDynError;

#[derive(Debug, Clone)]
pub struct MsSqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: Option<String>,
}

impl Default for MsSqlConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MsSqlConnectOptions {
    pub fn new() -> Self {
        Self {
            port: 1433,
            host: String::from("localhost"),
            username: String::from("sa"),
            password: None,
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
}

impl FromStr for MsSqlConnectOptions {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

        Ok(options)
    }
}
