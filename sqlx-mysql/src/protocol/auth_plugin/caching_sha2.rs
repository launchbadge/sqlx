use super::rsa::encrypt as rsa_encrypt;
use super::xor_eq;
use crate::protocol::AuthPlugin;
use crate::MySqlClientError;
use bytes::buf::Chain;
use bytes::Bytes;
use sha2::{Digest, Sha256};
use sqlx_core::Result;

/// Implements SHA-256 authentication but uses caching on the server-side for better performance.
/// After the first authentication, a fast path is used that doesn't involve the RSA key exchange.
///
/// https://dev.mysql.com/doc/refman/8.0/en/caching-sha2-pluggable-authentication.html
/// https://mariadb.com/kb/en/caching_sha2_password-authentication-plugin/
///
#[derive(Debug)]
pub(crate) struct CachingSha2AuthPlugin;

impl AuthPlugin for CachingSha2AuthPlugin {
    fn name(&self) -> &'static str {
        "caching_sha2_password"
    }

    fn invoke(&self, nonce: &Chain<Bytes, Bytes>, password: &str) -> Vec<u8> {
        if password.is_empty() {
            // empty password => no scramble
            return vec![];
        }

        // SHA256( password ) ^ SHA256( nonce + SHA256( SHA256( password ) ) )

        let mut hasher = Sha256::new();

        hasher.update(password);

        // SHA256( password )
        let mut pw_sha2 = hasher.finalize_reset();

        hasher.update(&pw_sha2);

        // SHA256( SHA256( password ) )
        let pw_sha2_sha2 = hasher.finalize_reset();

        hasher.update(pw_sha2_sha2);
        hasher.update(nonce.first_ref());
        hasher.update(nonce.last_ref());

        // SHA256( nonce + SHA256( SHA256( password ) ) )
        let nonce_pw_sha1_sha1 = hasher.finalize();

        xor_eq(&mut pw_sha2, &nonce_pw_sha1_sha1);

        pw_sha2.to_vec()
    }

    fn handle(
        &self,
        command: u8,
        data: Bytes,
        nonce: &Chain<Bytes, Bytes>,
        password: &str,
    ) -> Result<Option<Vec<u8>>> {
        const AUTH_SUCCESS: u8 = 0x3;
        const AUTH_CONTINUE: u8 = 0x4;

        if command != 0x01 {
            return Err(MySqlClientError::auth_plugin(
                self,
                format!("received 0x{:x} but expected 0x1 (MORE DATA)", command),
            )
            .into());
        }

        match data[0] {
            // good to go, return nothing
            AUTH_SUCCESS => Ok(None),

            AUTH_CONTINUE => {
                // ruh roh, we need to ask for the RSA public key, so we can
                // encrypt our password directly and send it
                Ok(Some(vec![0x2_u8]))
            }

            _ => {
                let rsa_pub_key = data;
                let encrypted = rsa_encrypt(self, &rsa_pub_key, password, nonce)?;

                Ok(Some(encrypted))
            }
        }
    }
}
