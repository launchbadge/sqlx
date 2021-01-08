use bytes::buf::Chain;
use bytes::Bytes;
use sqlx_core::Result;

/// Implements SHA-256 authentication.
///
/// Each time we connect we have to do an RSA key exchange.
/// This slows down auth quite a bit.
///
/// https://dev.mysql.com/doc/refman/8.0/en/sha256-pluggable-authentication.html
/// https://mariadb.com/kb/en/sha256_password-plugin/
///
#[derive(Debug)]
pub(crate) struct Sha256AuthPlugin;

impl super::AuthPlugin for Sha256AuthPlugin {
    fn name(&self) -> &'static str {
        "sha256_password"
    }

    fn invoke(&self, _nonce: &Chain<Bytes, Bytes>, password: &str) -> Vec<u8> {
        if password.is_empty() {
            // no password => do not ask for RSA key
            return vec![];
        }

        // ask for the RSA key
        vec![0x01]
    }

    fn handle(
        &self,
        data: Bytes,
        nonce: &Chain<Bytes, Bytes>,
        password: &str,
    ) -> Result<Option<Vec<u8>>> {
        let rsa_pub_key = data;
        let encrypted = super::rsa::encrypt(self.name(), &rsa_pub_key, password, nonce)?;

        Ok(Some(encrypted))
    }
}
