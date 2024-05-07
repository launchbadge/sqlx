use futures_util::future;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::ClientConnection;
use std::io::{self, BufReader, Cursor, Read, Write};
use std::sync::Arc;
use std::task::{Context, Poll};

use rustls::{
    CertificateError, ClientConfig, 
    Error as TlsError,
    client::WebPkiServerVerifier,
    RootCertStore,
};

use crate::error::Error;
use crate::io::ReadBuf;
use crate::net::tls::util::StdSocket;
use crate::net::tls::TlsConfig;
use crate::net::Socket;

pub struct RustlsSocket<S: Socket> {
    inner: StdSocket<S>,
    state: rustls::ClientConnection,
    close_notify_sent: bool,
}

impl<S: Socket> RustlsSocket<S> {
    fn poll_complete_io(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        loop {
            match self.state.complete_io(&mut self.inner) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    futures_util::ready!(self.inner.poll_ready(cx))?;
                }
                ready => return Poll::Ready(ready.map(|_| ())),
            }
        }
    }

    async fn complete_io(&mut self) -> io::Result<()> {
        future::poll_fn(|cx| self.poll_complete_io(cx)).await
    }
}

impl<S: Socket> Socket for RustlsSocket<S> {
    fn try_read(&mut self, buf: &mut dyn ReadBuf) -> io::Result<usize> {
        self.state.reader().read(buf.init_mut())
    }

    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.state.writer().write(buf) {
            // Returns a zero-length write when the buffer is full.
            Ok(0) => Err(io::ErrorKind::WouldBlock.into()),
            other => other,
        }
    }

    fn poll_read_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_complete_io(cx)
    }

    fn poll_write_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_complete_io(cx)
    }

    fn poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_complete_io(cx)
    }

    fn poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if !self.close_notify_sent {
            self.state.send_close_notify();
            self.close_notify_sent = true;
        }

        futures_util::ready!(self.poll_complete_io(cx))?;

        // Server can close socket as soon as it receives the connection shutdown request.
        // We shouldn't expect it to stick around for the TLS session to close cleanly.
        // https://security.stackexchange.com/a/82034
        let _ = futures_util::ready!(self.inner.socket.poll_shutdown(cx));

        Poll::Ready(Ok(()))
    }
}

pub async fn handshake<S>(socket: S, tls_config: TlsConfig<'_>) -> Result<RustlsSocket<S>, Error>
where
    S: Socket,
{
    let builder = ClientConfig::builder();

    // authentication using user's key and its associated certificate
    let user_auth = match (tls_config.client_cert_path, tls_config.client_key_path) {
        (Some(cert_path), Some(key_path)) => {
            let cert_chain = certs_from_pem(cert_path.data().await?)?;
            let key_der = private_key_from_pem(key_path.data().await?)?;
            Some((cert_chain, key_der))
        }
        (None, None) => None,
        (_, _) => {
            return Err(Error::Configuration(
                "user auth key and certs must be given together".into(),
            ))
        }
    };

    let config = if tls_config.accept_invalid_certs {
        if let Some(user_auth) = user_auth {
            builder
                .dangerous().with_custom_certificate_verifier(
                    Arc::new(DummyTlsVerifier)
                )
                .with_client_auth_cert(user_auth.0, user_auth.1)
                .map_err(Error::tls)?
        } else {
            builder
                .dangerous().with_custom_certificate_verifier(
                    Arc::new(DummyTlsVerifier)
                )
                .with_no_client_auth()
        }
    } else {
        let mut cert_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect()
        };

        if let Some(ca) = tls_config.root_cert_path {
            let mut cursor = Cursor::new(ca.data().await?);

            let mut errors = rustls_pemfile::certs(&mut cursor).filter_map(|cert_result| {
                match cert_result {
                    Err(_e) => {
                        Some(Error::Tls(format!("Invalid certificate {ca}").into()))
                    },
                    Ok(v) => {
                        match cert_store.add(v) {
                            Err(e) => Some(Error::Tls(e.into())),
                            Ok(()) => None
                        }
                    }
                }
            });
            // Return the first error
            if let Some(e) = errors.next() {
                return Err(e)
            }
        }

        if tls_config.accept_invalid_hostnames {
            let verifier = WebPkiServerVerifier::builder(
                Arc::new(cert_store)
            ).build()?;

            if let Some(user_auth) = user_auth {
                builder.dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoHostnameTlsVerifier { verifier }))
                    .with_client_auth_cert(user_auth.0, user_auth.1)
                    .map_err(Error::tls)?
            } else {
                builder.dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoHostnameTlsVerifier { verifier }))
                    .with_no_client_auth()
            }
        } else if let Some(user_auth) = user_auth {
            builder
                .with_root_certificates(cert_store)
                .with_client_auth_cert(user_auth.0, user_auth.1)
                .map_err(Error::tls)?
        } else {
            builder
                .with_root_certificates(cert_store)
                .with_no_client_auth()
        }
    };

    let host = ServerName::try_from(tls_config.hostname.to_string()).map_err(Error::tls)?;

    let mut socket = RustlsSocket {
        inner: StdSocket::new(socket),
        state: ClientConnection::new(Arc::new(config), host).map_err(Error::tls)?,
        close_notify_sent: false,
    };

    // Performs the TLS handshake or bails
    socket.complete_io().await?;

    Ok(socket)
}

fn certs_from_pem(pem: Vec<u8>) -> Result<Vec<CertificateDer<'static>>, Error> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);
    let mut err: Option<Error> = None;
    let mut certs: Vec<CertificateDer<'static>> = Vec::new();
    let mut certs_iter = rustls_pemfile::certs(&mut reader).into_iter();
    // Iterate over all results of rustls_pemfile::certs, collection them into a vec and
    // returning on the first error
    while let Some(cert_result) = certs_iter.next() {
        if err.is_none() {
            match cert_result {
                Err(e) => {
                    err = Some(Error::Io(e));
                },
                Ok(v) => {
                    certs.push(v);
                }
            }
        }
    };
    match err {
        Some(e) => Err(e),
        None => Ok(certs)
    }
}

fn private_key_from_pem(pem: Vec<u8>) -> Result<PrivateKeyDer<'static>, Error> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);

    loop {
        match rustls_pemfile::read_one(&mut reader)? {
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => {
                return Ok(PrivateKeyDer::Pkcs8(key))
            },
            Some(rustls_pemfile::Item::Sec1Key(key)) => {
                return Ok(PrivateKeyDer::Sec1(key))
            },
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => {
                return Ok(PrivateKeyDer::Pkcs1(key))
            },
            None => break,
            _ => {}
        }
    }

    Err(Error::Configuration("no keys found pem file".into()))
}

#[derive(Debug)]
struct DummyTlsVerifier;

impl ServerCertVerifier for DummyTlsVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec!()
    }
}

#[derive(Debug)]
pub struct NoHostnameTlsVerifier {
    verifier: Arc<WebPkiServerVerifier>,
}

impl ServerCertVerifier for NoHostnameTlsVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        intermediates: &[rustls::pki_types::CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        match self.verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        ) {
            Err(TlsError::InvalidCertificate(reason))
                if reason == CertificateError::NotValidForName =>
            {
                Ok(ServerCertVerified::assertion())
            }
            res => res,
        }
    }

    fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        self.verifier.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        self.verifier.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.verifier.supported_verify_schemes()
    }
}
