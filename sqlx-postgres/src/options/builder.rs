use std::mem;
use std::path::{Path, PathBuf};

use either::Either;

impl super::PgConnectOptions {
    /// Sets the hostname of the database server.
    ///
    /// If the hostname begins with a slash (`/`), it is interpreted as the absolute path
    /// to a Unix domain socket file instead of a hostname of a server.
    ///
    /// Defaults to either the `PGHOSTADDR` or `PGHOST` environment variable, falling back
    /// to `localhost` if neither are present.
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
    /// Overrides [`host`](#method.host).
    ///
    /// Defaults to, and overrides a default `host`, if one of the files is present in
    /// the local filesystem:
    ///
    /// -   `/var/run/postgresql/.s.PGSQL.{port}`
    /// -   `/private/tmp/.s.PGSQL.{port}`
    /// -   `/tmp/.s.PGSQL.{port}`
    ///
    pub fn socket(&mut self, socket: impl AsRef<Path>) -> &mut Self {
        self.address = Either::Right(socket.as_ref().to_owned());
        self
    }

    /// Sets the TCP port number of the database server.
    ///
    /// Defaults to the `PGPORT` environment variable, falling back to `5432`
    /// if not present.
    ///
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.address = match self.address {
            Either::Right(_) => Either::Left(("localhost".to_owned(), port)),
            Either::Left((ref mut host, _)) => Either::Left((mem::take(host), port)),
        };

        self
    }

    /// Sets the user to be used for authentication.
    ///
    /// Defaults to the `PGUSER` environment variable, if present.
    ///
    pub fn username(&mut self, username: impl AsRef<str>) -> &mut Self {
        self.username = Some(username.as_ref().to_owned());
        self
    }

    /// Sets the password to be used for authentication.
    ///
    /// Defaults to the `PGPASSWORD` environment variable, if present.
    ///
    pub fn password(&mut self, password: impl AsRef<str>) -> &mut Self {
        self.password = Some(password.as_ref().to_owned());
        self
    }

    /// Sets the database for the connection.
    ///
    /// Defaults to the `PGDATABASE` environment variable, falling back to
    /// the name of the user, if not present.
    ///
    pub fn database(&mut self, database: impl AsRef<str>) -> &mut Self {
        self.database = Some(database.as_ref().to_owned());
        self
    }

    /// Sets the application name for the connection.
    ///
    /// The name will be displayed in the `pg_stat_activity` view and
    /// included in CSV log entries. Only printable ASCII characters may be
    /// used in the `application_name` value.
    ///
    /// Defaults to the `PGAPPNAME` environment variable, if present.
    ///
    pub fn application_name(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.application_name = Some(name.as_ref().to_owned());
        self
    }
}
