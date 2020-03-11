use std::borrow::Cow;
use std::str::FromStr;

use crate::mysql::protocol::{Capabilities, SslRequest};
use crate::mysql::stream::MySqlStream;
use crate::url::Url;

pub(super) async fn upgrade_if_needed(stream: &mut MySqlStream, url: &Url) -> crate::Result<()> {
    let ca_file = url.param("ssl-ca");

    let ssl_mode = url.param("ssl-mode");

    let supports_tls = stream.capabilities.contains(Capabilities::SSL);

    // https://dev.mysql.com/doc/refman/5.7/en/connection-options.html#option_general_ssl-mode
    match ssl_mode.as_deref() {
        Some("DISABLED") => {}

        #[cfg(feature = "tls")]
        Some("PREFERRED") | None if !supports_tls => {}

        #[cfg(feature = "tls")]
        Some("PREFERRED") => {
            if let Err(error) = try_upgrade(stream, &url, None, true).await {
                // TLS upgrade failed; fall back to a normal connection
            }
        }

        #[cfg(feature = "tls")]
        Some(mode @ "REQUIRED") | Some(mode @ "VERIFY_CA") | Some(mode @ "VERIFY_IDENTITY")
            if !supports_tls =>
        {
            return Err(tls_err!("server does not support TLS").into());
        }

        #[cfg(feature = "tls")]
        Some(mode @ "VERIFY_CA") | Some(mode @ "VERIFY_IDENTITY") if ca_file.is_none() => {
            return Err(
                tls_err!("`ssl-mode` of {:?} requires `ssl-ca` to be set", ssl_mode).into(),
            );
        }

        #[cfg(feature = "tls")]
        Some(mode @ "REQUIRED") | Some(mode @ "VERIFY_CA") | Some(mode @ "VERIFY_IDENTITY") => {
            try_upgrade(
                stream,
                url,
                // false for both verify-ca and verify-full
                ca_file.as_deref(),
                // false for only verify-full
                mode != "VERIFY_IDENTITY",
            )
            .await?;
        }

        #[cfg(not(feature = "tls"))]
        None => {
            // The user neither explicitly enabled TLS in the connection string
            // nor did they turn the `tls` feature on
        }

        #[cfg(not(feature = "tls"))]
        Some(mode @ "PREFERRED")
        | Some(mode @ "REQUIRED")
        | Some(mode @ "VERIFY_CA")
        | Some(mode @ "VERIFY_IDENTITY") => {
            return Err(tls_err!(
                "ssl-mode {:?} unsupported; SQLx was compiled without `tls` feature",
                mode
            )
            .into());
        }

        Some(mode) => {
            return Err(tls_err!("unknown `ssl-mode` value: {:?}", mode).into());
        }
    }

    Ok(())
}

#[cfg(feature = "tls")]
async fn try_upgrade(
    stream: &mut MySqlStream,
    url: &Url,
    ca_file: Option<&str>,
    accept_invalid_hostnames: bool,
) -> crate::Result<()> {
    use crate::runtime::fs;

    use async_native_tls::{Certificate, TlsConnector};

    let mut connector = TlsConnector::new()
        .danger_accept_invalid_certs(ca_file.is_none())
        .danger_accept_invalid_hostnames(accept_invalid_hostnames);

    if let Some(ca_file) = ca_file {
        let root_cert = fs::read(ca_file).await?;

        connector = connector.add_root_certificate(Certificate::from_pem(&root_cert)?);
    }

    // send upgrade request and then immediately try TLS handshake
    stream
        .send(
            SslRequest {
                client_collation: COLLATE_UTF8MB4_UNICODE_CI,
                max_packet_size: MAX_PACKET_SIZE,
            },
            false,
        )
        .await?;

    stream.stream.upgrade(url, connector).await
}
