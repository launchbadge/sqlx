use std::iter::FromIterator;

use crate::error::Error;

use hmac::{Hmac, Mac, NewMac};
use md5::Digest;
use sha1::Sha1;
use sha2::Sha256;

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;

const SERVER_KEY: &[u8] = b"Sever Key";
const CLIENT_KEY: &[u8] = b"Client Key";
const DERIVE_KEY_ROUNDS: u32 = 2048;

fn derive_key_pbkdf2<T: AsRef<[u8]>>(
    password: T,
    salt_hex: &[u8],
    rounds: Option<u32>,
) -> Result<Vec<u8>, Error> {
    let salt = hex::decode(salt_hex).map_err(Error::protocol)?;
    let mut res = vec![0; 32];
    pbkdf2::pbkdf2::<HmacSha1>(
        password.as_ref(),
        &salt,
        rounds.unwrap_or(DERIVE_KEY_ROUNDS),
        &mut res,
    );

    Ok(res)
}

pub fn rfc5802_algo<T: AsRef<[u8]>>(
    password: T,
    random64code: &[u8; 64],
    token: &[u8; 8],
    iter: u32,
) -> Result<Vec<u8>, Error> {
    let key = derive_key_pbkdf2(password, random64code, Some(iter))?;
    let mut mac = HmacSha256::new_from_slice(&key).map_err(Error::protocol)?;

    let server_key = {
        mac.update(SERVER_KEY);
        mac.finalize_reset().into_bytes()
    };

    let client_key = {
        mac.update(CLIENT_KEY);
        mac.finalize_reset().into_bytes()
    };

    let stored_key = {
        let mut hasher = Sha256::new();
        hasher.update(&client_key);
        hasher.finalize()
    };

    let raw_token = hex::decode(token).map_err(Error::protocol)?;

    // fixme(kamx)
    let _client_signature = {
        let mut mac = HmacSha256::new_from_slice(&server_key).map_err(Error::protocol)?;
        mac.update(&raw_token);
        mac.finalize().into_bytes()
    };

    let hmac_result = {
        let mut mac = HmacSha256::new_from_slice(&stored_key).map_err(Error::protocol)?;
        mac.update(&raw_token);
        mac.finalize().into_bytes()
    };

    debug_assert!(hmac_result.len() == client_key.len());

    let mut result = Vec::from_iter(hmac_result);

    for i in 0..result.len() {
        result[i] ^= client_key[i];
    }

    Ok(hex::encode(&result).into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RANDOM64_CODE: &[u8] =
        b"6532663163366534343634313436333139373232393031653830313134396637";

    #[test]
    fn test_derive_key_pbkdf2() {
        const EXPECTED_KEY: [u8; 32] = [
            232u8, 187, 51, 118, 194, 38, 174, 58, 255, 34, 251, 193, 234, 144, 131, 194, 47, 15,
            228, 52, 178, 81, 148, 177, 29, 124, 166, 92, 24, 104, 59, 85,
        ];
        let actual_key = derive_key_pbkdf2(b"123", RANDOM64_CODE, None).unwrap();

        assert_eq!(&EXPECTED_KEY[..], &actual_key[..]);
    }

    #[test]
    fn test_rfc5802_algo() {
        let ranom64_code = [
            0x32, 0x33, 0x37, 0x36, 0x37, 0x33, 0x62, 0x33, 0x65, 0x66, 0x34, 0x62, 0x64, 0x65,
            0x32, 0x32, 0x64, 0x64, 0x61, 0x65, 0x61, 0x65, 0x63, 0x34, 0x61, 0x37, 0x36, 0x32,
            0x37, 0x31, 0x37, 0x32, 0x61, 0x35, 0x33, 0x62, 0x33, 0x34, 0x33, 0x38, 0x38, 0x32,
            0x30, 0x66, 0x35, 0x61, 0x64, 0x39, 0x35, 0x62, 0x30, 0x66, 0x30, 0x65, 0x66, 0x35,
            0x33, 0x66, 0x37, 0x35, 0x32, 0x32, 0x33, 0x64,
        ];

        let token = [0x64u8, 0x66, 0x61, 0x36, 0x32, 0x30, 0x30, 0x64];

        let x = rfc5802_algo("password", &ranom64_code, &token, 10000).unwrap();

        let expected = [
            0x63, 0x64, 0x66, 0x33, 0x66, 0x63, 0x61, 0x31, 0x37, 0x32, 0x34, 0x37, 0x65, 0x30,
            0x33, 0x34, 0x61, 0x64, 0x34, 0x61, 0x63, 0x63, 0x38, 0x36, 0x39, 0x37, 0x31, 0x32,
            0x31, 0x37, 0x36, 0x39, 0x39, 0x62, 0x61, 0x34, 0x37, 0x66, 0x38, 0x38, 0x38, 0x33,
            0x33, 0x30, 0x34, 0x63, 0x32, 0x35, 0x36, 0x61, 0x31, 0x33, 0x61, 0x61, 0x35, 0x63,
            0x37, 0x65, 0x31, 0x66, 0x36, 0x35, 0x37, 0x39,
        ];

        assert_eq!(&x, &expected);
    }
}
