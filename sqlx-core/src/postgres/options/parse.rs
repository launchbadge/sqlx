use crate::error::Error;
use crate::error::Error::ParseUrlError;
use crate::postgres::PgConnectOptions;
use std::mem;
use std::net::IpAddr;
use std::str::FromStr;

impl FromStr for PgConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let options = PgConnectOptions::new_without_pgpass();
        UrlParser::parse(s, options)
    }
}

struct UrlParser<'a> {
    s: &'a str,
}

impl<'a> UrlParser<'a> {
    // postgresql://[user[:password]@][netloc][:port][/dbname][?param1=value1&...]
    fn parse(s: &'a str, mut options: PgConnectOptions) -> Result<PgConnectOptions, Error> {
        let s = Self::remove_url_prefix(s)?;
        let mut parser = UrlParser { s };
        options = parser.parse_credentials(options)?;
        options = parser.parse_host(options)?;
        options = parser.parse_path(options)?;
        options = parser.parse_params(options)?;

        Ok(options)
    }

    fn remove_url_prefix(s: &str) -> Result<&str, Error> {
        for prefix in &["postgres://", "postgresql://"] {
            if let Some(stripped) = s.strip_prefix(prefix) {
                return Ok(stripped);
            }
        }
        Err(ParseUrlError(
            "The url prefix format is incorrect. Expect `postgres://` or `postgresql://`"
                .to_string(),
        ))
    }

    fn take_until(&mut self, end: &[char]) -> Option<&'a str> {
        match self.s.find(end) {
            Some(pos) => {
                let (head, tail) = self.s.split_at(pos);
                self.s = tail;
                Some(head)
            }
            None => None,
        }
    }

    fn take_all(&mut self) -> &'a str {
        mem::take(&mut self.s)
    }

    fn eat_byte(&mut self) {
        self.s = &self.s[1..];
    }

    fn parse_credentials(
        &mut self,
        mut option: PgConnectOptions,
    ) -> Result<PgConnectOptions, Error> {
        if let Some(cred) = self.take_until(&['@']) {
            let cred: Vec<&str> = cred.split(':').collect();
            if cred.len().gt(&1) {
                option = option.username(cred[0]);
                option = option.password(cred[1]);
            } else {
                option = option.username(cred[0]);
            }
        }
        self.eat_byte();
        Ok(option)
    }

    fn parse_host(&mut self, mut option: PgConnectOptions) -> Result<PgConnectOptions, Error> {
        let host = match self.take_until(&['/', '?']) {
            Some(host) => host,
            None => self.take_all(),
        };
        if host.is_empty() {
            return Ok(option);
        }

        let mut hosts = Vec::new();
        let mut ports = Vec::new();
        for chunk in host.split(',') {
            let (host, port) = if chunk.starts_with('[') {
                let idx = match chunk.find(']') {
                    Some(idx) => idx,
                    None => {
                        return Err(ParseUrlError(
                            "Incorrect url address, expect '[netloc]:port,.. `".to_string(),
                        ))
                    }
                };

                let host = &chunk[1..idx];
                let remaining = &chunk[idx + 1..];
                let port = if let Some(port) = remaining.strip_prefix(':') {
                    Some(port)
                } else if remaining.is_empty() {
                    None
                } else {
                    return Err(ParseUrlError("Incorrect url address, there is no port after the colon, expect '[netloc]:port,.. `".to_string()));
                };

                (host, port)
            } else {
                let mut it = chunk.splitn(2, ':');
                (it.next().unwrap(), it.next())
            };

            hosts.push(host.to_string());
            let port = port.unwrap_or("5432");
            ports.push(port.parse().map_err(Error::config)?);
        }
        option = option.host(hosts);
        option = option.port(ports);
        Ok(option)
    }

    fn parse_path(&mut self, mut option: PgConnectOptions) -> Result<PgConnectOptions, Error> {
        if !self.s.starts_with('/') {
            return Ok(option);
        }
        self.eat_byte();

        let dbname = match self.take_until(&['?']) {
            Some(dbname) => dbname,
            None => self.take_all(),
        };

        if !dbname.is_empty() {
            option = option.database(dbname);
        }

        Ok(option)
    }

    fn parse_params(&mut self, mut option: PgConnectOptions) -> Result<PgConnectOptions, Error> {
        if !self.s.starts_with('?') {
            return Ok(option);
        }
        self.eat_byte();

        while !self.s.is_empty() {
            let key = match self.take_until(&['=']) {
                Some(key) => key,
                None => return Err(ParseUrlError("Incorrect url address, expected: `postgresql://[user[:password]@][netloc][:port][/dbname][?param1=value1&...]`".to_string())),
            };
            self.eat_byte();

            let value = match self.take_until(&['&']) {
                Some(value) => {
                    self.eat_byte();
                    value
                }
                None => self.take_all(),
            };

            match &*key {
                "sslmode" | "ssl-mode" => {
                    option = option.ssl_mode(value.parse().map_err(Error::config)?);
                }

                "sslrootcert" | "ssl-root-cert" | "ssl-ca" => {
                    option = option.ssl_root_cert(&*value);
                }

                "statement-cache-capacity" => {
                    option = option.statement_cache_capacity(value.parse().map_err(Error::config)?);
                }

                "host" => {
                    if value.starts_with("/") {
                        option = option.socket(&*value);
                    } else {
                        option = option.host(vec![&*value]);
                    }
                }

                "hostaddr" => {
                    value.parse::<IpAddr>().map_err(Error::config)?;
                    option = option.host(vec![&*value])
                }

                "port" => option = option.port(vec![value.parse().map_err(Error::config)?]),

                "dbname" => option = option.database(&*value),

                "user" => option = option.username(&*value),

                "password" => option = option.password(&*value),

                "application_name" => option = option.application_name(&*value),

                "options" => {
                    if let Some(options) = option.options.as_mut() {
                        options.push(' ');
                        options.push_str(&*value);
                    } else {
                        option.options = Some(value.to_string());
                    }
                }

                k if k.starts_with("options[") => {
                    if let Some(key) = k.strip_prefix("options[").unwrap().strip_suffix(']') {
                        option = option.options([(key, &*value)]);
                    }
                }

                "target_session_attrs" => {
                    option = option.target_session_attrs(value.parse().map_err(Error::config)?)
                }

                _ => log::warn!("ignoring unrecognized connect parameter: {}={}", key, value),
            }
        }

        Ok(option)
    }
}

#[test]
fn it_parses_socket_correctly_from_parameter() {
    let url = "postgres:///?host=/var/run/postgres/";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("/var/run/postgres/".into()), opts.socket);
}

#[test]
fn it_parses_host_correctly_from_parameter() {
    let url = "postgres:///?host=google.database.com";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!("google.database.com", &opts.host);
}

#[test]
fn it_parses_hostaddr_correctly_from_parameter() {
    let url = "postgres:///?hostaddr=8.8.8.8";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!("8.8.8.8", &opts.host);
}

#[test]
fn it_parses_port_correctly_from_parameter() {
    let url = "postgres:///?port=1234";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!(1234, opts.port);
}

#[test]
fn it_parses_dbname_correctly_from_parameter() {
    let url = "postgres:///?dbname=some_db";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!(Some("some_db"), opts.database.as_deref());
}

#[test]
fn it_parses_user_correctly_from_parameter() {
    let url = "postgres:///?user=some_user";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!("some_user", opts.username);
}

#[test]
fn it_parses_password_correctly_from_parameter() {
    let url = "postgres:///?password=some_pass";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(None, opts.socket);
    assert_eq!(Some("some_pass"), opts.password.as_deref());
}

#[test]
fn it_parses_application_name_correctly_from_parameter() {
    let url = "postgres:///?application_name=some_name";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("some_name"), opts.application_name.as_deref());
}

#[test]
fn it_parses_username_with_at_sign_correctly() {
    let url = "postgres://user@hostname:password@hostname:5432/database";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!("user@hostname", &opts.username);
}

#[test]
fn it_parses_password_with_non_ascii_chars_correctly() {
    let url = "postgres://username:p@ssw0rd@hostname:5432/database";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("p@ssw0rd".into()), opts.password);
}

#[test]
fn it_parses_socket_correctly_percent_encoded() {
    let url = "postgres://%2Fvar%2Flib%2Fpostgres/database";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("/var/lib/postgres/".into()), opts.socket);
}
#[test]
fn it_parses_socket_correctly_with_username_percent_encoded() {
    let url = "postgres://some_user@%2Fvar%2Flib%2Fpostgres/database";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!("some_user", opts.username);
    assert_eq!(Some("/var/lib/postgres/".into()), opts.socket);
    assert_eq!(Some("database"), opts.database.as_deref());
}
#[test]
fn it_parses_libpq_options_correctly() {
    let url = "postgres:///?options=-c%20synchronous_commit%3Doff%20--search_path%3Dpostgres";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(
        Some("-c synchronous_commit=off --search_path=postgres".into()),
        opts.options
    );
}
#[test]
fn it_parses_sqlx_options_correctly() {
    let url = "postgres:///?options[synchronous_commit]=off&options[search_path]=postgres";
    let opts = PgConnectOptions::from_str(url).unwrap();

    assert_eq!(
        Some("-c synchronous_commit=off -c search_path=postgres".into()),
        opts.options
    );
}
