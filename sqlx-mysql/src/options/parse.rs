use std::borrow::Cow;
use std::str::FromStr;

use percent_encoding::percent_decode_str;
use sqlx_core::Error;
use url::Url;

use crate::MySqlConnectOptions;

impl FromStr for MySqlConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: Url = s.parse().map_err(|error| Error::opt("for database URL", error))?;

        if !matches!(url.scheme(), "mysql") {
            return Err(Error::opt_msg(format!(
                "unsupported URL scheme {:?} for MySQL",
                url.scheme()
            )));
        }

        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options.host(percent_decode_str_utf8(host));
        }

        if let Some(port) = url.port() {
            options.port(port);
        }

        let username = url.username();
        if !username.is_empty() {
            options.username(percent_decode_str_utf8(username));
        }

        if let Some(password) = url.password() {
            options.password(percent_decode_str_utf8(password));
        }

        let mut path = url.path();

        if path.starts_with('/') {
            path = &path[1..];
        }

        if !path.is_empty() {
            options.database(path);
        }

        for (key, value) in url.query_pairs() {
            match &*key {
                "user" | "username" => {
                    options.username(value);
                }

                "password" => {
                    options.password(value);
                }

                "ssl-mode" | "sslmode" | "sslMode" | "tls" => {
                    todo!()
                }

                "charset" => {
                    options.charset(value);
                }

                "timezone" => {
                    options.timezone(value);
                }

                "socket" => {
                    options.socket(&*value);
                }

                _ => {
                    // ignore unknown connection parameters
                    // fixme: should we error or warn here?
                }
            }
        }

        Ok(options)
    }
}

fn percent_decode_str_utf8(value: &str) -> Cow<'_, str> {
    percent_decode_str(value).decode_utf8_lossy()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::MySqlConnectOptions;

    #[test]
    fn parse() {
        let url = "mysql://user:password@hostname:5432/database?timezone=system&charset=utf8";
        let options: MySqlConnectOptions = url.parse().unwrap();

        assert_eq!(options.get_username(), Some("user"));
        assert_eq!(options.get_password(), Some("password"));
        assert_eq!(options.get_host(), "hostname");
        assert_eq!(options.get_port(), 5432);
        assert_eq!(options.get_database(), Some("database"));
        assert_eq!(options.get_timezone(), "system");
        assert_eq!(options.get_charset(), "utf8");
    }

    #[test]
    fn parse_with_defaults() {
        let url = "mysql://";
        let options: MySqlConnectOptions = url.parse().unwrap();

        assert_eq!(options.get_username(), None);
        assert_eq!(options.get_password(), None);
        assert_eq!(options.get_host(), "localhost");
        assert_eq!(options.get_port(), 3306);
        assert_eq!(options.get_database(), None);
        assert_eq!(options.get_timezone(), "utc");
        assert_eq!(options.get_charset(), "utf8mb4");
    }

    #[test]
    fn parse_socket_from_query() {
        let url = "mysql://user:password@localhost/database?socket=/var/run/mysqld/mysqld.sock";
        let options: MySqlConnectOptions = url.parse().unwrap();

        assert_eq!(options.get_username(), Some("user"));
        assert_eq!(options.get_password(), Some("password"));
        assert_eq!(options.get_database(), Some("database"));
        assert_eq!(options.get_socket(), Some(Path::new("/var/run/mysqld/mysqld.sock")));
    }

    #[test]
    fn parse_socket_from_host() {
        // socket path in host requires URL encoding - but does work
        let url = "mysql://user:password@%2Fvar%2Frun%2Fmysqld%2Fmysqld.sock/database";
        let options: MySqlConnectOptions = url.parse().unwrap();

        assert_eq!(options.get_username(), Some("user"));
        assert_eq!(options.get_password(), Some("password"));
        assert_eq!(options.get_database(), Some("database"));
        assert_eq!(options.get_socket(), Some(Path::new("/var/run/mysqld/mysqld.sock")));
    }

    #[test]
    #[should_panic]
    fn fail_to_parse_non_mysql() {
        let url = "postgres://user:password@hostname:5432/database?timezone=system&charset=utf8";
        let _: MySqlConnectOptions = url.parse().unwrap();
    }

    #[test]
    fn parse_username_with_at_sign() {
        let url = "mysql://user@hostname:password@hostname:5432/database";
        let options: MySqlConnectOptions = url.parse().unwrap();

        assert_eq!(options.get_username(), Some("user@hostname"));
    }

    #[test]
    fn parse_password_with_non_ascii_chars() {
        let url = "mysql://username:p@ssw0rd@hostname:5432/database";
        let options: MySqlConnectOptions = url.parse().unwrap();

        assert_eq!(options.get_password(), Some("p@ssw0rd"));
    }
}
