use crate::postgres::protocol::SslRequest;
use crate::postgres::PgConnection;
use crate::url::Url;

impl PgConnection {
    #[cfg(feature = "tls")]
    pub(super) async fn try_ssl(
        &mut self,
        url: &Url,
        invalid_certs: bool,
        invalid_hostnames: bool,
    ) -> crate::Result<bool> {
        use async_native_tls::TlsConnector;

        SslRequest::encode(self.stream.buffer_mut());

        self.stream.flush().await?;

        match self.stream.peek(1).await? {
            Some(b"N") => return Ok(false),
            Some(b"S") => (),
            Some(other) => {
                return Err(tls_err!("unexpected single-byte response: 0x{:02X}", other[0]).into())
            }
            None => return Err(tls_err!("server unexpectedly closed connection").into()),
        }

        let mut connector = TlsConnector::new()
            .danger_accept_invalid_certs(invalid_certs)
            .danger_accept_invalid_hostnames(invalid_hostnames);

        if !invalid_certs {
            match read_root_certificate(&url).await {
                Ok(cert) => {
                    connector = connector.add_root_certificate(cert);
                }
                Err(e) => log::warn!("failed to read Postgres root certificate: {}", e),
            }
        }

        self.stream.clear_bufs();
        self.stream.stream.upgrade(url, connector).await?;

        Ok(true)
    }
}

#[cfg(feature = "tls")]
async fn read_root_certificate(url: &Url) -> crate::Result<async_native_tls::Certificate> {
    use std::env;

    let root_cert_path = if let Some(path) = url.get_param("sslrootcert") {
        path.into()
    } else if let Ok(cert_path) = env::var("PGSSLROOTCERT") {
        cert_path
    } else if cfg!(windows) {
        let appdata = env::var("APPDATA").map_err(|_| tls_err!("APPDATA not set"))?;
        format!("{}\\postgresql\\root.crt", appdata)
    } else {
        let home = env::var("HOME").map_err(|_| tls_err!("HOME not set"))?;
        format!("{}/.postgresql/root.crt", home)
    };

    let root_cert = crate::runtime::fs::read(root_cert_path).await?;
    Ok(async_native_tls::Certificate::from_pem(&root_cert)?)
}
