use bytes::{buf::Chain, Bytes};
use sha1::{Digest, Sha1};
use sqlx_core::Result;

use super::xor_eq;

// https://mariadb.com/kb/en/connection/#mysql_native_password-plugin
// https://dev.mysql.com/doc/internals/en/secure-password-authentication.html

#[derive(Debug)]
pub(crate) struct NativeAuthPlugin;

impl super::AuthPlugin for NativeAuthPlugin {
    fn name(&self) -> &'static str {
        "mysql_native_password"
    }

    fn invoke(&self, nonce: &Chain<Bytes, Bytes>, password: &str) -> Vec<u8> {
        if password.is_empty() {
            // no password => empty scramble
            return vec![];
        }

        // SHA1( password ) ^ SHA1( nonce + SHA1( SHA1( password ) ) )

        let mut hasher = Sha1::new();

        hasher.update(password);

        // SHA1( password )
        let mut pw_sha1 = hasher.finalize_reset();

        hasher.update(&pw_sha1);

        // SHA1( SHA1( password ) )
        let pw_sha1_sha1 = hasher.finalize_reset();

        hasher.update(nonce.first_ref());
        hasher.update(&nonce.last_ref());
        hasher.update(&pw_sha1_sha1);

        // SHA1( seed + SHA1( SHA1( password ) ) )
        let nonce_pw_sha1_sha1 = hasher.finalize();

        xor_eq(&mut pw_sha1, &nonce_pw_sha1_sha1);

        pw_sha1.to_vec()
    }

    fn handle(
        &self,
        _data: Bytes,
        _nonce: &Chain<Bytes, Bytes>,
        _password: &str,
    ) -> Result<Option<Vec<u8>>> {
        // MySQL should not be returning any additional data for
        // the native mysql auth plugin
        unreachable!()
    }
}
