use std::path::Path;

use super::MySqlConnectOptions;
use crate::Runtime;

impl<Rt: Runtime> MySqlConnectOptions<Rt> {
    /// Returns the hostname of the database server.
    #[must_use]
    #[inline]
    pub fn get_host(&self) -> &str {
        self.0.get_host()
    }

    /// Returns the TCP port number of the database server.
    #[must_use]
    #[inline]
    pub fn get_port(&self) -> u16 {
        self.0.get_port()
    }

    /// Returns the path to the Unix domain socket, if one is configured.
    #[must_use]
    #[inline]
    pub fn get_socket(&self) -> Option<&Path> {
        self.0.get_socket()
    }

    /// Returns the default database name.
    #[must_use]
    #[inline]
    pub fn get_database(&self) -> Option<&str> {
        self.0.get_database()
    }

    /// Returns the username to be used for authentication.
    #[must_use]
    #[inline]
    pub fn get_username(&self) -> Option<&str> {
        self.0.get_username()
    }

    /// Returns the password to be used for authentication.
    #[must_use]
    #[inline]
    pub fn get_password(&self) -> Option<&str> {
        self.0.get_password()
    }

    /// Returns the character set for the connection.
    #[must_use]
    #[inline]
    pub fn get_charset(&self) -> &str {
        self.0.get_charset()
    }

    /// Returns the timezone for the connection.
    #[must_use]
    #[inline]
    pub fn get_timezone(&self) -> &str {
        self.0.get_timezone()
    }
}
