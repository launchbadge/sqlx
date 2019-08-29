use std::net::{IpAddr, SocketAddr};

pub struct Url(url::Url);

impl Url {
    pub fn parse(url: &str) -> Self {
        // TODO: Handle parse errors
        Url(url::Url::parse(url).unwrap())
    }

    pub fn host(&self) -> &str {
        self.0.host_str().unwrap_or("localhost")
    }

    pub fn port(&self, default: u16) -> u16 {
        self.0.port().unwrap_or(default)
    }

    pub fn address(&self, default_port: u16) -> SocketAddr {
        // TODO: DNS
        let host: IpAddr = self.host().parse().unwrap();
        let port = self.port(default_port);

        (host, port).into()
    }

    pub fn username(&self) -> &str {
        self.0.username()
    }

    pub fn password(&self) -> Option<&str> {
        self.0.password()
    }

    pub fn database(&self) -> &str {
        self.0.path().trim_start_matches('/')
    }
}
