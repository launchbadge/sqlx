use std::mem;
use std::path::{Path, PathBuf};

use either::Either;
use sqlx_core::Runtime;

impl super::MySqlConnectOptions {
    /// Sets the hostname of the database server.
    ///
    /// If the hostname begins with a slash (`/`), it is interpreted as the absolute path
    /// to a Unix domain socket file instead of a hostname of a server.
    ///
    /// Defaults to `localhost`.
    ///
    pub fn host(&mut self, host: impl AsRef<str>) -> &mut Self {
        let host = host.as_ref();

        self.address = if host.starts_with('/') {
            Either::Right(PathBuf::from(&*host))
        } else {
            Either::Left((host.into(), self.get_port()))
        };

        self
    }

    /// Sets the path of the Unix domain socket to connect to.
    ///
    /// Overrides [`host()`](#method.host) and [`port()`](#method.port).
    ///
    pub fn socket(&mut self, socket: impl AsRef<Path>) -> &mut Self {
        self.address = Either::Right(socket.as_ref().to_owned());
        self
    }

    /// Sets the TCP port number of the database server.
    ///
    /// Defaults to `3306`.
    ///
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.address = match self.address {
            Either::Right(_) => Either::Left(("localhost".to_owned(), port)),
            Either::Left((ref mut host, _)) => Either::Left((mem::take(host), port)),
        };

        self
    }

    /// Sets the username to be used for authentication.
    // FIXME: Specify what happens when you do NOT set this
    pub fn username(&mut self, username: impl AsRef<str>) -> &mut Self {
        self.username = Some(username.as_ref().to_owned());
        self
    }

    /// Sets the password to be used for authentication.
    pub fn password(&mut self, password: impl AsRef<str>) -> &mut Self {
        self.password = Some(password.as_ref().to_owned());
        self
    }

    /// Sets the default database for the connection.
    pub fn database(&mut self, database: impl AsRef<str>) -> &mut Self {
        self.database = Some(database.as_ref().to_owned());
        self
    }

    /// Sets the character set for the connection.
    pub fn charset(&mut self, charset: impl AsRef<str>) -> &mut Self {
        self.charset = charset.as_ref().to_owned();
        self
    }

    /// Sets the timezone for the connection.
    pub fn timezone(&mut self, timezone: impl AsRef<str>) -> &mut Self {
        self.timezone = timezone.as_ref().to_owned();
        self
    }
}
