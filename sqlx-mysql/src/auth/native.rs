use bytes::{buf::Chain, Bytes};
use sha1::{Digest, Sha1};

use super::xor_eq;

// https://mariadb.com/kb/en/connection/#mysql_native_password-plugin
// https://dev.mysql.com/doc/internals/en/secure-password-authentication.html

pub(crate) fn scramble(nonce: &Chain<Bytes, Bytes>, password: &str) -> Vec<u8> {
    // SHA1( password ) ^ SHA1( nonce + SHA1( SHA1( password ) ) )

    let mut hasher = Sha1::new();

    hasher.update(password);

    // SHA1( password )
    let mut pw_sha1 = hasher.finalize_reset();

    hasher.update(&pw_sha1);

    // SHA1( SHA1( password ) )
    let pw_sha1_sha1 = hasher.finalize_reset();

    // NOTE: use the first 20 bytes of the nonce, we MAY have gotten a nul terminator
    hasher.update(nonce.first_ref());
    hasher.update(&nonce.last_ref()[..20 - nonce.first_ref().len()]);
    hasher.update(&pw_sha1_sha1);

    // SHA1( seed + SHA1( SHA1( password ) ) )
    let nonce_pw_sha1_sha1 = hasher.finalize();

    xor_eq(&mut pw_sha1, &nonce_pw_sha1_sha1);

    pw_sha1.to_vec()
}
