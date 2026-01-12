use std::io::{self, Read, Write};

use crate::io::ReadBuf;
use crate::net::tls::util::StdSocket;
use crate::net::tls::RawTlsConfig;
use crate::net::tls::TlsConfig;
use crate::net::Socket;
use crate::rt;
use crate::Error;

use native_tls::{HandshakeError, Identity};
use std::task::{Context, Poll};

pub struct NativeTlsSocket<S: Socket> {
    stream: native_tls::TlsStream<StdSocket<S>>,
}

impl<S: Socket> Socket for NativeTlsSocket<S> {
    fn try_read(&mut self, buf: &mut dyn ReadBuf) -> io::Result<usize> {
        self.stream.read(buf.init_mut())
    }

    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn poll_read_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.stream.get_mut().poll_ready(cx)
    }

    fn poll_write_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.stream.get_mut().poll_ready(cx)
    }

    fn poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.stream.shutdown() {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => self.stream.get_mut().poll_ready(cx),
            ready => Poll::Ready(ready),
        }
    }
}

impl TlsConfig<'_> {
    async fn native_tls_connector(&self) -> crate::Result<(native_tls::TlsConnector, &str), Error> {
        #[allow(irrefutable_let_patterns)]
        let TlsConfig::RawTlsConfig(RawTlsConfig {
            root_cert,
            client_cert,
            client_key,
            accept_invalid_certs,
            accept_invalid_hostnames,
            hostname,
        }) = self
        else {
            unreachable!()
        };
        let mut builder = native_tls::TlsConnector::builder();

        builder
            .danger_accept_invalid_certs(*accept_invalid_certs)
            .danger_accept_invalid_hostnames(*accept_invalid_hostnames);

        if let Some(root_cert) = root_cert {
            let data = root_cert.data().await?;
            builder.add_root_certificate(
                native_tls::Certificate::from_pem(&data).map_err(Error::tls)?,
            );
        }

        // authentication using user's key-file and its associated certificate
        if let (Some(cert), Some(key)) = (client_cert, client_key) {
            let cert = cert.data().await?;
            let key = key.data().await?;
            let identity = Identity::from_pkcs8(&cert, &key).map_err(Error::tls)?;
            builder.identity(identity);
        }

        // The openssl TlsConnector synchronously loads certificates from files.
        // Loading these files can block for tens of milliseconds.
        let connector = rt::spawn_blocking(move || builder.build())
            .await
            .map_err(Error::tls)?;
        Ok((connector, hostname))
    }
}

pub async fn handshake<S: Socket>(
    socket: S,
    config: TlsConfig<'_>,
) -> crate::Result<NativeTlsSocket<S>> {
    let (connector, hostname) = config.native_tls_connector().await?;
    let mut mid_handshake = match connector.connect(hostname, StdSocket::new(socket)) {
        Ok(tls_stream) => return Ok(NativeTlsSocket { stream: tls_stream }),
        Err(HandshakeError::Failure(e)) => return Err(Error::tls(e)),
        Err(HandshakeError::WouldBlock(mid_handshake)) => mid_handshake,
    };

    loop {
        mid_handshake.get_mut().ready().await?;

        match mid_handshake.handshake() {
            Ok(tls_stream) => return Ok(NativeTlsSocket { stream: tls_stream }),
            Err(HandshakeError::Failure(e)) => return Err(Error::tls(e)),
            Err(HandshakeError::WouldBlock(mid_handshake_)) => {
                mid_handshake = mid_handshake_;
            }
        }
    }
}
