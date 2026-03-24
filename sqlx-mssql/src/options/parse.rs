use std::str::FromStr;

use percent_encoding::percent_decode_str;
use sqlx_core::Url;

use crate::error::Error;

use super::ssl_mode::MssqlSslMode;
use super::MssqlConnectOptions;

impl MssqlConnectOptions {
    pub(crate) fn parse_from_url(url: &Url) -> Result<Self, Error> {
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
                &percent_decode_str(username)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        if let Some(password) = url.password() {
            options = options.password(
                &percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(
                &percent_decode_str(path)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        for (key, value) in url.query_pairs() {
            match &*key {
                "sslmode" | "ssl_mode" => {
                    options = options.ssl_mode(match &*value {
                        "disabled" => MssqlSslMode::Disabled,
                        "login_only" => MssqlSslMode::LoginOnly,
                        "preferred" => MssqlSslMode::Preferred,
                        "required" => MssqlSslMode::Required,
                        _ => {
                            return Err(Error::Configuration(
                                format!("unknown sslmode value: {value}").into(),
                            ))
                        }
                    });
                }

                "encrypt" => {
                    options = options.encrypt(value.parse().map_err(Error::config)?);
                }

                "trust_server_certificate" | "trustServerCertificate" => {
                    options =
                        options.trust_server_certificate(value.parse().map_err(Error::config)?);
                }

                "instance" => {
                    options = options.instance(&value);
                }

                "app_name" | "application-name" => {
                    options = options.app_name(&value);
                }

                "statement-cache-capacity" => {
                    options =
                        options.statement_cache_capacity(value.parse().map_err(Error::config)?);
                }

                "application_intent" | "applicationIntent" => match &*value {
                    "read_only" | "ReadOnly" => {
                        options = options.application_intent_read_only(true);
                    }
                    "read_write" | "ReadWrite" => {
                        options = options.application_intent_read_only(false);
                    }
                    _ => {
                        return Err(Error::Configuration(
                            format!("unknown application_intent value: {value}").into(),
                        ))
                    }
                },

                "trust_server_certificate_ca" | "trustServerCertificateCa" => {
                    options = options.trust_server_certificate_ca(&value);
                }

                "auth" => {
                    match &*value {
                        "sql_server" => {}
                        #[cfg(all(windows, feature = "winauth"))]
                        "windows" => {
                            options.windows_auth = true;
                        }
                        #[cfg(any(
                            all(windows, feature = "winauth"),
                            all(unix, feature = "integrated-auth-gssapi")
                        ))]
                        "integrated" => {
                            options.integrated_auth = true;
                        }
                        "aad_token" => {
                            // token value is set via the separate `token` parameter
                        }
                        _ => {
                            return Err(Error::Configuration(
                                format!("unknown auth value: {value}").into(),
                            ))
                        }
                    }
                }

                "token" => {
                    options.aad_token = Some(value.into_owned());
                }

                _ => {}
            }
        }

        Ok(options)
    }

    pub(crate) fn build_url(&self) -> Result<Url, Error> {
        let mut url = Url::parse(&format!(
            "mssql://{}@{}:{}",
            self.username, self.host, self.port
        ))
        .map_err(|e| Error::Configuration(e.to_string().into()))?;

        if let Some(password) = &self.password {
            let _ = url.set_password(Some(password));
        }

        if let Some(database) = &self.database {
            url.set_path(database);
        }

        let sslmode = match self.ssl_mode {
            MssqlSslMode::Disabled => "disabled",
            MssqlSslMode::LoginOnly => "login_only",
            MssqlSslMode::Preferred => "preferred",
            MssqlSslMode::Required => "required",
        };
        url.query_pairs_mut().append_pair("sslmode", sslmode);

        if self.application_intent_read_only {
            url.query_pairs_mut()
                .append_pair("application_intent", "read_only");
        }

        if let Some(ca_path) = &self.trust_server_certificate_ca {
            url.query_pairs_mut()
                .append_pair("trust_server_certificate_ca", ca_path);
        }

        if let Some(token) = &self.aad_token {
            url.query_pairs_mut()
                .append_pair("auth", "aad_token")
                .append_pair("token", token);
        } else {
            #[cfg(any(
                all(windows, feature = "winauth"),
                all(unix, feature = "integrated-auth-gssapi")
            ))]
            if self.integrated_auth {
                url.query_pairs_mut().append_pair("auth", "integrated");
            }

            #[cfg(all(windows, feature = "winauth"))]
            if self.windows_auth && !self.integrated_auth {
                url.query_pairs_mut().append_pair("auth", "windows");
            }
        }

        Ok(url)
    }
}

impl FromStr for MssqlConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse().map_err(Error::config)?;
        Self::parse_from_url(&url)
    }
}

#[test]
fn it_parses_basic_mssql_url() {
    let url = "mssql://sa:password@localhost:1433/master";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(opts.host, "localhost");
    assert_eq!(opts.port, 1433);
    assert_eq!(opts.username, "sa");
    assert_eq!(opts.password, Some("password".into()));
    assert_eq!(opts.database, Some("master".into()));
}

#[test]
fn it_parses_url_with_instance() {
    let url = "mssql://sa:password@localhost/master?instance=SQLEXPRESS";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(opts.instance, Some("SQLEXPRESS".into()));
}

#[test]
fn it_parses_sslmode_disabled() {
    let url = "mssql://sa:password@localhost/master?sslmode=disabled";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(matches!(opts.ssl_mode, MssqlSslMode::Disabled));
}

#[test]
fn it_parses_sslmode_login_only() {
    let url = "mssql://sa:password@localhost/master?ssl_mode=login_only";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(matches!(opts.ssl_mode, MssqlSslMode::LoginOnly));
}

#[test]
fn it_parses_sslmode_preferred() {
    let url = "mssql://sa:password@localhost/master?sslmode=preferred";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(matches!(opts.ssl_mode, MssqlSslMode::Preferred));
}

#[test]
fn it_parses_sslmode_required() {
    let url = "mssql://sa:password@localhost/master?sslmode=required";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(matches!(opts.ssl_mode, MssqlSslMode::Required));
}

#[test]
fn it_parses_encrypt_true_as_required() {
    let url = "mssql://sa:password@localhost/master?encrypt=true";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(matches!(opts.ssl_mode, MssqlSslMode::Required));
}

#[test]
fn it_parses_encrypt_false_as_disabled() {
    let url = "mssql://sa:password@localhost/master?encrypt=false";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(matches!(opts.ssl_mode, MssqlSslMode::Disabled));
}

#[test]
fn it_rejects_invalid_sslmode() {
    let url = "mssql://sa:password@localhost/master?sslmode=bogus";
    assert!(MssqlConnectOptions::from_str(url).is_err());
}

#[test]
fn it_roundtrips_sslmode_in_url() {
    let url = "mssql://sa:password@localhost/master?sslmode=login_only";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    let built = opts.build_url().unwrap();
    let opts2 = MssqlConnectOptions::parse_from_url(&built).unwrap();
    assert!(matches!(opts2.ssl_mode, MssqlSslMode::LoginOnly));
}

#[test]
fn it_parses_application_intent_read_only() {
    let url = "mssql://sa:password@localhost/master?application_intent=read_only";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(opts.application_intent_read_only);
}

#[test]
fn it_parses_application_intent_read_write() {
    let url = "mssql://sa:password@localhost/master?application_intent=read_write";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(!opts.application_intent_read_only);
}

#[test]
fn it_parses_application_intent_camel_case() {
    let url = "mssql://sa:password@localhost/master?applicationIntent=ReadOnly";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert!(opts.application_intent_read_only);
}

#[test]
fn it_rejects_invalid_application_intent() {
    let url = "mssql://sa:password@localhost/master?application_intent=bogus";
    assert!(MssqlConnectOptions::from_str(url).is_err());
}

#[test]
fn it_parses_trust_server_certificate_ca() {
    let url = "mssql://sa:password@localhost/master?trust_server_certificate_ca=/path/to/ca.pem";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!(
        opts.trust_server_certificate_ca,
        Some("/path/to/ca.pem".into())
    );
}

#[test]
fn it_roundtrips_application_intent_in_url() {
    let opts = MssqlConnectOptions::new()
        .host("localhost")
        .username("sa")
        .password("password")
        .application_intent_read_only(true);
    let built = opts.build_url().unwrap();
    let opts2 = MssqlConnectOptions::parse_from_url(&built).unwrap();
    assert!(opts2.application_intent_read_only);
}

#[test]
fn it_roundtrips_trust_cert_ca_in_url() {
    let opts = MssqlConnectOptions::new()
        .host("localhost")
        .username("sa")
        .password("password")
        .trust_server_certificate_ca("/etc/ssl/ca.pem");
    let built = opts.build_url().unwrap();
    let opts2 = MssqlConnectOptions::parse_from_url(&built).unwrap();
    assert_eq!(
        opts2.trust_server_certificate_ca,
        Some("/etc/ssl/ca.pem".into())
    );
}

#[test]
fn it_parses_aad_token_auth() {
    let url = "mssql://sa@localhost/master?auth=aad_token&token=my-bearer-token";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!(opts.aad_token, Some("my-bearer-token".into()));
}

#[test]
fn it_roundtrips_aad_token_in_url() {
    let opts = MssqlConnectOptions::new()
        .host("localhost")
        .username("sa")
        .aad_token("my-bearer-token");
    let built = opts.build_url().unwrap();
    let opts2 = MssqlConnectOptions::parse_from_url(&built).unwrap();
    assert_eq!(opts2.aad_token, Some("my-bearer-token".into()));
}

#[test]
fn it_parses_sql_server_auth_explicitly() {
    let url = "mssql://sa:password@localhost/master?auth=sql_server";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!(opts.aad_token, None);
}

#[test]
fn it_rejects_invalid_auth() {
    let url = "mssql://sa:password@localhost/master?auth=bogus";
    assert!(MssqlConnectOptions::from_str(url).is_err());
}
