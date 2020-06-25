use bytes::Bytes;
use sqlx_rt::{
    fs,
    native_tls::{Certificate, TlsConnector},
};

use crate::error::Error;
use crate::postgres::connection::stream::PgStream;
use crate::postgres::message::SslRequest;
use crate::postgres::{PgConnectOptions, PgSslMode};

pub(super) async fn maybe_upgrade(
    stream: &mut PgStream,
    options: &PgConnectOptions,
) -> Result<(), Error> {
    // https://www.postgresql.org/docs/12/libpq-ssl.html#LIBPQ-SSL-SSLMODE-STATEMENTS
    match options.ssl_mode {
        // FIXME: Implement ALLOW
        PgSslMode::Allow | PgSslMode::Disable => {}

        PgSslMode::Prefer => {
            // try upgrade, but its okay if we fail
            upgrade(stream, options).await?;
        }

        PgSslMode::Require | PgSslMode::VerifyFull | PgSslMode::VerifyCa => {
            if !upgrade(stream, options).await? {
                // upgrade failed, die
                return Err(Error::Tls("server does not support TLS".into()));
            }
        }
    }

    Ok(())
}

async fn upgrade(stream: &mut PgStream, options: &PgConnectOptions) -> Result<bool, Error> {
    // https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.11

    // To initiate an SSL-encrypted connection, the frontend initially sends an
    // SSLRequest message rather than a StartupMessage

    stream.send(SslRequest).await?;

    // The server then responds with a single byte containing S or N, indicating that
    // it is willing or unwilling to perform SSL, respectively.

    match stream.read::<Bytes>(1).await?[0] {
        b'S' => {
            // The server is ready and willing to accept an SSL connection
        }

        b'N' => {
            // The server is _unwilling_ to perform SSL
            return Ok(false);
        }

        other => {
            return Err(err_protocol!(
                "unexpected response from SSLRequest: 0x{:02x}",
                other
            ));
        }
    }

    // FIXME: de-duplicate with mysql/connection/tls.rs

    let accept_invalid_certs = !matches!(
        options.ssl_mode,
        PgSslMode::VerifyCa | PgSslMode::VerifyFull
    );

    let mut builder = TlsConnector::builder();
    builder
        .danger_accept_invalid_certs(accept_invalid_certs)
        .danger_accept_invalid_hostnames(!matches!(options.ssl_mode, PgSslMode::VerifyFull));

    if !accept_invalid_certs {
        if let Some(ca) = &options.ssl_root_cert {
            let data = fs::read(ca).await?;
            let cert = Certificate::from_pem(&data).map_err(Error::tls)?;

            builder.add_root_certificate(cert);
        }
    }

    #[cfg(not(feature = "runtime-async-std"))]
    let connector = builder.build().map_err(Error::tls)?;

    #[cfg(feature = "runtime-async-std")]
    let connector = builder;

    stream.upgrade(&options.host, connector.into()).await?;

    Ok(true)
}
