use std::str::from_utf8;

use crate::protocol::auth_plugin::xor_eq;
use crate::protocol::AuthPlugin;
use crate::MySqlClientError;
use bytes::buf::Chain;
use bytes::Bytes;
use rsa::{PaddingScheme, PublicKey, RSAPublicKey};
use sqlx_core::Result;

pub(crate) fn encrypt(
    plugin: &impl AuthPlugin,
    key: &[u8],
    password: &str,
    nonce: &Chain<Bytes, Bytes>,
) -> Result<Vec<u8>> {
    // xor the password with the given nonce
    let mut pass = to_asciz(password);

    let (a, b) = (nonce.first_ref(), nonce.last_ref());
    let mut nonce = Vec::with_capacity(a.len() + b.len());

    nonce.extend_from_slice(&*a);
    nonce.extend_from_slice(&*b);

    xor_eq(&mut pass, &*nonce);

    // client sends an RSA encrypted password
    let public = parse_rsa_pub_key(plugin, key)?;
    let padding = PaddingScheme::new_oaep::<sha1::Sha1>();

    public
        .encrypt(&mut rng(), padding, &pass[..])
        .map_err(|err| MySqlClientError::auth_plugin(plugin, err).into())
}

// https://docs.rs/rsa/0.3.0/rsa/struct.RSAPublicKey.html?search=#example-1
fn parse_rsa_pub_key(plugin: &impl AuthPlugin, key: &[u8]) -> Result<RSAPublicKey> {
    let key = from_utf8(key).map_err(|err| MySqlClientError::auth_plugin(plugin, err))?;

    // Takes advantage of the knowledge that we know
    // we are receiving a PKCS#8 RSA Public Key at all
    // times from MySQL

    let encoded =
        key.lines().filter(|line| !line.starts_with('-')).fold(String::new(), |mut data, line| {
            data.push_str(line);
            data
        });

    let der = base64::decode(&encoded).map_err(|err| MySqlClientError::auth_plugin(plugin, err))?;

    RSAPublicKey::from_pkcs8(&der).map_err(|err| MySqlClientError::auth_plugin(plugin, err).into())
}

fn to_asciz(s: &str) -> Vec<u8> {
    let mut z = String::with_capacity(s.len() + 1);
    z.push_str(s);
    z.push('\0');

    z.into_bytes()
}

// use a stable stream of numbers for encryption
// during tests to assert the result of [encrypt]

#[cfg(not(test))]
fn rng() -> rand::rngs::ThreadRng {
    rand::thread_rng()
}

#[cfg(test)]
fn rng() -> rand::rngs::mock::StepRng {
    rand::rngs::mock::StepRng::new(0, 1)
}
