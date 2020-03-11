use crate::mysql::stream::MySqlStream;
use crate::url::Url;

#[cfg_attr(not(feature = "tls"), allow(unused_variables))]
pub(super) async fn upgrade_if_needed(stream: &mut MySqlStream, url: &Url) -> crate::Result<()> {
    #[cfg_attr(not(feature = "tls"), allow(unused_imports))]
    use crate::mysql::protocol::Capabilities;

    let ca_file = url.param("ssl-ca");
    let ssl_mode = url.param("ssl-mode");

    // https://dev.mysql.com/doc/refman/5.7/en/connection-options.html#option_general_ssl-mode
    match ssl_mode.as_deref() {
        Some("DISABLED") => {}

        #[cfg(feature = "tls")]
        Some("PREFERRED") | None if !stream.capabilities.contains(Capabilities::SSL) => {}

        #[cfg(feature = "tls")]
        Some("PREFERRED") => {
            if let Err(_error) = try_upgrade(stream, &url, None, true).await {
                // TLS upgrade failed; fall back to a normal connection
            }
        }

        #[cfg(feature = "tls")]
        None => {
            if let Err(_error) = try_upgrade(stream, &url, ca_file.as_deref(), true).await {
                // TLS upgrade failed; fall back to a normal connection
            }
        }

        #[cfg(feature = "tls")]
        Some("REQUIRED") | Some("VERIFY_CA") | Some("VERIFY_IDENTITY")
            if !stream.capabilities.contains(Capabilities::SSL) =>
        {
            return Err(tls_err!("server does not support TLS").into());
        }

        #[cfg(feature = "tls")]
        Some("VERIFY_CA") | Some("VERIFY_IDENTITY") if ca_file.is_none() => {
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
    use crate::mysql::protocol::SslRequest;
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
                client_collation: super::connection::COLLATE_UTF8MB4_UNICODE_CI,
                max_packet_size: super::connection::MAX_PACKET_SIZE,
            },
            false,
        )
        .await?;

    stream.stream.upgrade(url, connector).await
}
