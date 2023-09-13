use crate::error::Error;
use crate::mssql::MssqlConnectOptions;
use percent_encoding::percent_decode_str;
use std::str::FromStr;
use url::Url;

impl FromStr for MssqlConnectOptions {
    type Err = Error;

    /// Parse a connection string into a set of connection options.
    ///
    /// The connection string is expected to be a valid URL with the following format:
    /// ```text
    /// mssql://[username[:password]@]host/database[?instance=instance_name&packet_size=packet_size&client_program_version=client_program_version&client_pid=client_pid&hostname=hostname&app_name=app_name&server_name=server_name&client_interface_name=client_interface_name&language=language]
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: Url = s.parse().map_err(Error::config)?;
        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options = options.host(host);
        }

        if let Some(port) = url.port() {
            options = options.port(port);
        }

        let username = url.username();
        if !username.is_empty() {
            options = options.username(
                &*percent_decode_str(username)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        if let Some(password) = url.password() {
            options = options.password(
                &*percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(path);
        }

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "instance" => {
                    options = options.instance(&*value);
                }
                "packet_size" => {
                    let size = value.parse().map_err(Error::config)?;
                    options = options
                        .requested_packet_size(size)
                        .map_err(|_| Error::config(MssqlInvalidOption(format!("packet_size={}", size))))?;
                }
                "client_program_version" => {
                    options = options.client_program_version(value.parse().map_err(Error::config)?)
                }
                "client_pid" => options = options.client_pid(value.parse().map_err(Error::config)?),
                "hostname" => options = options.hostname(&*value),
                "app_name" => options = options.app_name(&*value),
                "server_name" => options = options.server_name(&*value),
                "client_interface_name" => options = options.client_interface_name(&*value),
                "language" => options = options.language(&*value),
                _ => {
                    return Err(Error::config(MssqlInvalidOption(key.into())));
                }
            }
        }
        Ok(options)
    }
}

#[derive(Debug)]
struct MssqlInvalidOption(String);

impl std::fmt::Display for MssqlInvalidOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` is not a valid mssql connection option", self.0)
    }
}

impl std::error::Error for MssqlInvalidOption {}

#[test]
fn it_parses_username_with_at_sign_correctly() {
    let url = "mysql://user@hostname:password@hostname:5432/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!("user@hostname", &opts.username);
}

#[test]
fn it_parses_password_with_non_ascii_chars_correctly() {
    let url = "mysql://username:p@ssw0rd@hostname:5432/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("p@ssw0rd".into()), opts.password);
}
