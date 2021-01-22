use std::path::Path;

use super::PostgresConnectOptions;
use crate::Runtime;

impl<Rt: Runtime> PostgresConnectOptions<Rt> {
    /// Sets the hostname of the database server.
    ///
    /// If the hostname begins with a slash (`/`), it is interpreted as the absolute path
    /// to a Unix domain socket file instead of a hostname of a server.
    ///
    /// Defaults to `localhost`.
    ///
    #[inline]
    pub fn host(&mut self, host: impl AsRef<str>) -> &mut Self {
        self.0.host(host);
        self
    }

    /// Sets the path of the Unix domain socket to connect to.
    ///
    /// Overrides [`host()`](#method.host) and [`port()`](#method.port).
    ///
    #[inline]
    pub fn socket(&mut self, socket: impl AsRef<Path>) -> &mut Self {
        self.0.socket(socket);
        self
    }

    /// Sets the TCP port number of the database server.
    ///
    /// Defaults to `3306`.
    ///
    #[inline]
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.0.port(port);
        self
    }

    /// Sets the username to be used for authentication.
    // FIXME: Specify what happens when you do NOT set this
    pub fn username(&mut self, username: impl AsRef<str>) -> &mut Self {
        self.0.username(username);
        self
    }

    /// Sets the password to be used for authentication.
    #[inline]
    pub fn password(&mut self, password: impl AsRef<str>) -> &mut Self {
        self.0.password(password);
        self
    }

    /// Sets the default database for the connection.
    #[inline]
    pub fn database(&mut self, database: impl AsRef<str>) -> &mut Self {
        self.0.database(database);
        self
    }

    /// Sets the character set for the connection.
    #[inline]
    pub fn charset(&mut self, charset: impl AsRef<str>) -> &mut Self {
        self.0.charset(charset);
        self
    }

    /// Sets the timezone for the connection.
    #[inline]
    pub fn timezone(&mut self, timezone: impl AsRef<str>) -> &mut Self {
        self.0.timezone(timezone);
        self
    }
}
