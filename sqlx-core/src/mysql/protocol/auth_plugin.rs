use digest::{Digest, FixedOutput};
use generic_array::GenericArray;
use memchr::memchr;
use sha1::Sha1;
use sha2::Sha256;

use crate::mysql::util::xor_eq;
use crate::mysql::MySql;

#[derive(Debug, PartialEq)]
pub enum AuthPlugin {
    MySqlNativePassword,
    CachingSha2Password,
    Sha256Password,
}

impl AuthPlugin {
    pub(crate) fn from_opt_str(s: Option<&str>) -> crate::Result<MySql, AuthPlugin> {
        match s {
            Some("mysql_native_password") | None => Ok(AuthPlugin::MySqlNativePassword),
            Some("caching_sha2_password") => Ok(AuthPlugin::CachingSha2Password),
            Some("sha256_password") => Ok(AuthPlugin::Sha256Password),

            Some(s) => {
                Err(protocol_err!("requires unimplemented authentication plugin: {}", s).into())
            }
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AuthPlugin::MySqlNativePassword => "mysql_native_password",
            AuthPlugin::CachingSha2Password => "caching_sha2_password",
            AuthPlugin::Sha256Password => "sha256_password",
        }
    }

    pub(crate) fn scramble(&self, password: &str, nonce: &[u8]) -> Vec<u8> {
        match self {
            AuthPlugin::MySqlNativePassword => {
                // The [nonce] for mysql_native_password is (optionally) nul terminated
                let end = memchr(b'\0', nonce).unwrap_or(nonce.len());

                scramble_sha1(password, &nonce[..end]).to_vec()
            }
            AuthPlugin::CachingSha2Password => scramble_sha256(password, nonce).to_vec(),

            _ => unimplemented!(),
        }
    }
}

fn scramble_sha1(
    password: &str,
    seed: &[u8],
) -> GenericArray<u8, <Sha1 as FixedOutput>::OutputSize> {
    // SHA1( password ) ^ SHA1( seed + SHA1( SHA1( password ) ) )
    // https://mariadb.com/kb/en/connection/#mysql_native_password-plugin

    let mut ctx = Sha1::new();

    ctx.input(password);

    let mut pw_hash = ctx.result_reset();

    ctx.input(&pw_hash);

    let pw_hash_hash = ctx.result_reset();

    ctx.input(seed);
    ctx.input(pw_hash_hash);

    let pw_seed_hash_hash = ctx.result();

    xor_eq(&mut pw_hash, &pw_seed_hash_hash);

    pw_hash
}

fn scramble_sha256(
    password: &str,
    seed: &[u8],
) -> GenericArray<u8, <Sha256 as FixedOutput>::OutputSize> {
    // XOR(SHA256(password), SHA256(seed, SHA256(SHA256(password))))
    // https://mariadb.com/kb/en/caching_sha2_password-authentication-plugin/#sha-2-encrypted-password
    let mut ctx = Sha256::new();

    ctx.input(password);

    let mut pw_hash = ctx.result_reset();

    ctx.input(&pw_hash);

    let pw_hash_hash = ctx.result_reset();

    ctx.input(seed);
    ctx.input(pw_hash_hash);

    let pw_seed_hash_hash = ctx.result();

    xor_eq(&mut pw_hash, &pw_seed_hash_hash);

    pw_hash
}
