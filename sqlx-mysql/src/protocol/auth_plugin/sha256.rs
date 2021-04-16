use bytes::buf::Chain;
use bytes::Bytes;
use sqlx_core::Result;

use super::rsa::encrypt as rsa_encrypt;
use crate::protocol::AuthPlugin;
use crate::MySqlClientError;

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

impl AuthPlugin for Sha256AuthPlugin {
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
        command: u8,
        data: Bytes,
        nonce: &Chain<Bytes, Bytes>,
        password: &str,
    ) -> Result<Option<Vec<u8>>> {
        if command != 0x01 {
            return Err(MySqlClientError::auth_plugin(
                self,
                format!("Received 0x{:x} but expected 0x1 (MORE DATA)", command),
            )
            .into());
        }

        let rsa_pub_key = data;
        let encrypted = rsa_encrypt(self, &rsa_pub_key, password, nonce)?;

        Ok(Some(encrypted))
    }
}
