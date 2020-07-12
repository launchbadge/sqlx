use crate::error::Error;
use crate::postgres::PgConnectOptions;
use std::str::FromStr;
use url::Url;

impl FromStr for PgConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse().map_err(Error::config)?;

        let mut options = Self::default();

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

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(path);
        }

        for (key, value) in url.query_pairs().into_iter() {
            match &*key {
                "sslmode" | "ssl-mode" => {
                    options = options.ssl_mode(value.parse().map_err(Error::config)?);
                }

                "sslrootcert" | "ssl-root-cert" | "ssl-ca" => {
                    options = options.ssl_root_cert(&*value);
                }

                "statement-cache-capacity" => {
                    options =
                        options.statement_cache_capacity(value.parse().map_err(Error::config)?);
                }

                "host" => {
                    if value.starts_with("/") {
                        options = options.socket(&*value);
                    } else {
                        options = options.host(&*value);
                    }
                }

                _ => {}
            }
        }

        Ok(options)
    }
}

#[test]
fn it_parses_socket_correctly_from_parameter() {
    let uri = "postgres:///?host=/var/run/postgres/";
    let opts = PgConnectOptions::from_str(uri).unwrap();

    assert_eq!(Some("/var/run/postgres/".into()), opts.socket);
}

#[test]
fn it_parses_host_correctly_from_parameter() {
    let uri = "postgres:///?host=google.database.com";
    let opts = PgConnectOptions::from_str(uri).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!("google.database.com", &opts.host);
}
