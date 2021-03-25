use std::env::var;
use std::path::{Path, PathBuf};

use either::Either;

use crate::PgConnectOptions;

pub(crate) const HOST: &str = "localhost";
pub(crate) const PORT: u16 = 5432;

impl Default for PgConnectOptions {
    fn default() -> Self {
        let port = var("PGPORT").ok().and_then(|v| v.parse().ok()).unwrap_or(PORT);

        let mut self_ = Self {
            address: default_address(port),
            username: var("PGUSER").ok(),
            password: var("PGPASSWORD").ok(),
            database: var("PGDATABASE").ok(),
            application_name: var("PGAPPNAME").ok(),
        };

        if let Some(host) = var("PGHOSTADDR").ok().or_else(|| var("PGHOST").ok()) {
            // apply PGHOST down here to let someone set a socket
            // path via PGHOST
            self_.host(&host);
        }

        self_
    }
}

impl PgConnectOptions {
    /// Creates a default set of options ready for configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

fn default_address(port: u16) -> Either<(String, u16), PathBuf> {
    // try to check for the existence of a unix socket and uses that
    let socket = format!(".s.PGSQL.{}", port);
    let candidates = [
        "/var/run/postgresql", // Debian
        "/private/tmp",        // OSX (homebrew)
        "/tmp",                // Default
    ];

    for candidate in &candidates {
        if Path::new(candidate).join(&socket).exists() {
            return Either::Right(PathBuf::from(candidate));
        }
    }

    Either::Left((HOST.to_owned(), port))
}
