use crate::net::CertificateInput;
use rustls::{
    Certificate, ClientConfig, RootCertStore, ServerCertVerified, ServerCertVerifier, TLSError,
    WebPKIVerifier,
};
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use webpki::DNSNameRef;

use crate::error::Error;

pub async fn configure_tls_connector(
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    root_cert_path: Option<&CertificateInput>,
    client_cert_path: Option<&CertificateInput>,
    client_key_path: Option<&CertificateInput>,
) -> Result<sqlx_rt::TlsConnector, Error> {
    let mut config = ClientConfig::new();

    if accept_invalid_certs {
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(DummyTlsVerifier));
    } else {
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        if let Some(ca) = root_cert_path {
            let data = ca.data().await?;
            let mut cursor = Cursor::new(data);
            config
                .root_store
                .add_pem_file(&mut cursor)
                .map_err(|_| Error::Tls(format!("Invalid certificate {}", ca).into()))?;
        }

        if let (Some(cert), Some(key)) = (client_cert_path, client_key_path) {
            let key_data = key.data().await?;
            let cert_data = cert.data().await?;
            let certs = to_certs(cert_data);
            let key = to_private_key(key_data)?;
            match config.set_single_client_cert(certs, key) {
                Ok(_) => (),
                Err(err) => {
                    return Err(Error::Configuration(
                        format!("no keys found in: {:?}", err).into(),
                    ))
                }
            }
        }

        if accept_invalid_hostnames {
            config
                .dangerous()
                .set_certificate_verifier(Arc::new(NoHostnameTlsVerifier));
        }
    }

    Ok(Arc::new(config).into())
}

struct DummyTlsVerifier;

impl ServerCertVerifier for DummyTlsVerifier {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        _presented_certs: &[Certificate],
        _dns_name: DNSNameRef<'_>,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

pub struct NoHostnameTlsVerifier;

impl ServerCertVerifier for NoHostnameTlsVerifier {
    fn verify_server_cert(
        &self,
        roots: &RootCertStore,
        presented_certs: &[Certificate],
        dns_name: DNSNameRef<'_>,
        ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        let verifier = WebPKIVerifier::new();
        match verifier.verify_server_cert(roots, presented_certs, dns_name, ocsp_response) {
            Err(TLSError::WebPKIError(webpki::Error::CertNotValidForName)) => {
                Ok(ServerCertVerified::assertion())
            }
            res => res,
        }
    }
}

fn to_certs(pem: Vec<u8>) -> Vec<rustls::Certificate> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn to_private_key(pem: Vec<u8>) -> Result<rustls::PrivateKey, Error> {
    let cur = Cursor::new(pem);
    let mut reader = BufReader::new(cur);

    loop {
        match rustls_pemfile::read_one(&mut reader)? {
            Some(rustls_pemfile::Item::RSAKey(key)) => return Ok(rustls::PrivateKey(key)),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return Ok(rustls::PrivateKey(key)),
            None => break,
            _ => {}
        }
    }

    Err(Error::Configuration("no keys found pem file".into()))
}
