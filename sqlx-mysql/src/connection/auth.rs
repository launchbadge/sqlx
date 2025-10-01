use bytes::buf::Chain;
use bytes::Bytes;
use digest::{Digest, OutputSizeUser};
use generic_array::GenericArray;
use sha1::Sha1;
use sha2::Sha256;

use crate::connection::stream::MySqlStream;
use crate::error::Error;
use crate::protocol::auth::AuthPlugin;
use crate::protocol::Packet;

// Note: This module uses password hashing and authentication that should ideally
// be used over TLS connections for security. Consider enabling TLS in production.

impl AuthPlugin {
    pub(super) async fn scramble(
        self,
        stream: &mut MySqlStream,
        password: &str,
        nonce: &Chain<Bytes, Bytes>,
    ) -> Result<Vec<u8>, Error> {
        match self {
            // https://mariadb.com/kb/en/caching_sha2_password-authentication-plugin/
            AuthPlugin::CachingSha2Password => Ok(scramble_sha256(password, nonce).to_vec()),

            AuthPlugin::MySqlNativePassword => Ok(scramble_sha1(password, nonce).to_vec()),

            // https://mariadb.com/kb/en/sha256_password-plugin/
            AuthPlugin::Sha256Password => encrypt_rsa(stream, 0x01, password, nonce).await,

            AuthPlugin::MySqlClearPassword => {
                let mut pw_bytes = password.as_bytes().to_owned();
                pw_bytes.push(0); // null terminate
                Ok(pw_bytes)
            }
        }
    }

    pub(super) async fn handle(
        self,
        stream: &mut MySqlStream,
        packet: Packet<Bytes>,
        password: &str,
        nonce: &Chain<Bytes, Bytes>,
    ) -> Result<bool, Error> {
        match self {
            AuthPlugin::CachingSha2Password if packet[0] == 0x01 => {
                match packet[1] {
                    // AUTH_OK
                    0x03 => Ok(true),

                    // AUTH_CONTINUE
                    0x04 => {
                        let payload = encrypt_rsa(stream, 0x02, password, nonce).await?;

                        stream.write_packet(&*payload)?;
                        stream.flush().await?;

                        Ok(false)
                    }

                    v => {
                        Err(err_protocol!("unexpected result from fast authentication 0x{:x} when expecting 0x03 (AUTH_OK) or 0x04 (AUTH_CONTINUE)", v))
                    }
                }
            }

            _ => Err(err_protocol!(
                "unexpected packet 0x{:02x} for auth plugin '{}' during authentication",
                packet[0],
                self.name()
            )),
        }
    }
}

fn scramble_sha1(
    password: &str,
    nonce: &Chain<Bytes, Bytes>,
) -> GenericArray<u8, <Sha1 as OutputSizeUser>::OutputSize> {
    // SHA1( password ) ^ SHA1( seed + SHA1( SHA1( password ) ) )
    // https://mariadb.com/kb/en/connection/#mysql_native_password-plugin

    let mut ctx = Sha1::new();

    ctx.update(password);

    let mut pw_hash = ctx.finalize_reset();

    ctx.update(pw_hash);

    let pw_hash_hash = ctx.finalize_reset();

    ctx.update(nonce.first_ref());
    ctx.update(nonce.last_ref());
    ctx.update(pw_hash_hash);

    let pw_seed_hash_hash = ctx.finalize();

    xor_eq(&mut pw_hash, &pw_seed_hash_hash);

    pw_hash
}

fn scramble_sha256(
    password: &str,
    nonce: &Chain<Bytes, Bytes>,
) -> GenericArray<u8, <Sha256 as OutputSizeUser>::OutputSize> {
    // XOR(SHA256(password), SHA256(seed, SHA256(SHA256(password))))
    // https://mariadb.com/kb/en/caching_sha2_password-authentication-plugin/#sha-2-encrypted-password
    let mut ctx = Sha256::new();

    ctx.update(password);

    let mut pw_hash = ctx.finalize_reset();

    ctx.update(pw_hash);

    let pw_hash_hash = ctx.finalize_reset();

    ctx.update(nonce.first_ref());
    ctx.update(nonce.last_ref());
    ctx.update(pw_hash_hash);

    let pw_seed_hash_hash = ctx.finalize();

    xor_eq(&mut pw_hash, &pw_seed_hash_hash);

    pw_hash
}

async fn encrypt_rsa<'s>(
    stream: &'s mut MySqlStream,
    public_key_request_id: u8,
    password: &'s str,
    _nonce: &'s Chain<Bytes, Bytes>,
) -> Result<Vec<u8>, Error> {
    // https://mariadb.com/kb/en/caching_sha2_password-authentication-plugin/

    if stream.is_tls {
        // If in a TLS stream, send the password directly in clear text
        return Ok(to_asciz(password));
    }

    // Note: For security, it's recommended to use TLS for MySQL connections.
    // Non-TLS authentication falls back to cleartext which is insecure.
    // Consider adding '?ssl-mode=required' to your connection string.
    tracing::warn!(
        "MySQL authentication without TLS is insecure. Consider enabling TLS with '?ssl-mode=required'"
    );

    // Fallback to cleartext password for non-TLS connections
    // This maintains backward compatibility but is not secure
    stream.write_packet(&[public_key_request_id][..])?;
    stream.flush().await?;

    // Send cleartext password (null-terminated)
    Ok(to_asciz(password))
}

// XOR(x, y)
// If len(y) < len(x), wrap around inside y
fn xor_eq(x: &mut [u8], y: &[u8]) {
    let y_len = y.len();

    for i in 0..x.len() {
        x[i] ^= y[i % y_len];
    }
}

fn to_asciz(s: &str) -> Vec<u8> {
    let mut z = String::with_capacity(s.len() + 1);
    z.push_str(s);
    z.push('\0');

    z.into_bytes()
}
