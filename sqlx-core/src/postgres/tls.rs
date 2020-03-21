use crate::postgres::database::Postgres;
use crate::postgres::stream::PgStream;
use crate::url::Url;

#[cfg_attr(not(feature = "tls"), allow(unused_variables))]
pub(crate) async fn request_if_needed(
    stream: &mut PgStream,
    url: &Url,
) -> crate::Result<Postgres, ()> {
    // https://www.postgresql.org/docs/12/libpq-ssl.html#LIBPQ-SSL-SSLMODE-STATEMENTS
    match url.param("sslmode").as_deref() {
        Some("disable") | Some("allow") => {
            // Do nothing
        }

        #[cfg(feature = "tls")]
        Some("prefer") | None => {
            // We default to [prefer] if TLS is compiled in
            if !try_upgrade(stream, url, true, true).await? {
                // TLS upgrade failed; fall back to a normal connection
            }
        }

        #[cfg(not(feature = "tls"))]
        None => {
            // The user neither explicitly enabled TLS in the connection string
            // nor did they turn the `tls` feature on

            // Do nothing
        }

        #[cfg(feature = "tls")]
        Some(mode @ "require") | Some(mode @ "verify-ca") | Some(mode @ "verify-full") => {
            if !try_upgrade(
                stream,
                url,
                // false for both verify-ca and verify-full
                mode == "require",
                // false for only verify-full
                mode != "verify-full",
            )
            .await?
            {
                return Err(tls_err!("server does not support TLS").into());
            }
        }

        #[cfg(not(feature = "tls"))]
        Some(mode @ "prefer")
        | Some(mode @ "require")
        | Some(mode @ "verify-ca")
        | Some(mode @ "verify-full") => {
            return Err(tls_err!(
                "sslmode {:?} unsupported; SQLx was compiled without `tls` feature",
                mode
            )
            .into());
        }

        Some(mode) => {
            return Err(tls_err!("unknown `sslmode` value: {:?}", mode).into());
        }
    }

    Ok(())
}

#[cfg(feature = "tls")]
async fn try_upgrade(
    stream: &mut PgStream,
    url: &Url,
    accept_invalid_certs: bool,
    accept_invalid_host_names: bool,
) -> crate::Result<Postgres, bool> {
    use async_native_tls::TlsConnector;

    stream.write(crate::postgres::protocol::SslRequest);
    stream.flush().await?;

    // The server then responds with a single byte containing S or N,
    // indicating that it is willing or unwilling to perform SSL, respectively.
    let ind = stream.stream.peek(1).await?[0];
    stream.stream.consume(1);

    match ind {
        b'S' => {
            // The server is ready and willing to accept an SSL connection
        }

        b'N' => {
            // The server is _unwilling_ to perform SSL
            return Ok(false);
        }

        other => {
            return Err(tls_err!("unexpected response from SSLRequest: 0x{:02X}", other).into());
        }
    }

    let mut connector = TlsConnector::new()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .danger_accept_invalid_hostnames(accept_invalid_host_names);

    if !accept_invalid_certs {
        // Try to read in the root certificate for postgres using several
        // standard methods (used by psql and libpq)
        if let Some(cert) = read_root_certificate(&url).await? {
            connector = connector.add_root_certificate(cert);
        }
    }

    stream.stream.upgrade(url, connector).await?;

    Ok(true)
}

#[cfg(feature = "tls")]
async fn read_root_certificate(
    url: &Url,
) -> crate::Result<Postgres, Option<async_native_tls::Certificate>> {
    use crate::runtime::fs;
    use std::env;

    let mut data = None;

    if let Some(path) = url
        .param("sslrootcert")
        .or_else(|| env::var("PGSSLROOTCERT").ok().map(Into::into))
    {
        data = Some(fs::read(&*path).await?);
    } else if cfg!(windows) {
        if let Ok(app_data) = env::var("APPDATA") {
            let path = format!("{}\\postgresql\\root.crt", app_data);

            data = fs::read(path).await.ok();
        }
    } else {
        if let Ok(home) = env::var("HOME") {
            let path = format!("{}/.postgresql/root.crt", home);

            data = fs::read(path).await.ok();
        }
    }

    data.map(|data| async_native_tls::Certificate::from_pem(&data))
        .transpose()
        .map_err(Into::into)
}
