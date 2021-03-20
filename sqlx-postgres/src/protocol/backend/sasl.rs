use std::convert::TryFrom;

use bytes::Bytes;
use bytestring::ByteString;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct AuthenticationSasl(Bytes);

impl Deserialize<'_> for AuthenticationSasl {
    fn deserialize_with(buf: Bytes, _: ()) -> Result<Self> {
        Ok(Self(buf))
    }
}

#[derive(Debug)]
pub(crate) struct AuthenticationSaslContinue {
    pub(crate) salt: Box<[u8]>,
    pub(crate) iterations: u32,
    pub(crate) nonce: ByteString,
    pub(crate) message: ByteString,
}

impl Deserialize<'_> for AuthenticationSaslContinue {
    fn deserialize_with(buf: Bytes, _: ()) -> Result<Self> {
        let mut iterations: u32 = 4096;
        let mut salt = Vec::new();
        let mut nonce = Bytes::new();

        // [Example]
        // r=/z+giZiTxAH7r8sNAeHr7cvpqV3uo7G/bJBIJO3pjVM7t3ng,s=4UV68bIkC8f9/X8xH7aPhg==,i=4096

        for item in buf.split(|b| *b == b',') {
            let key = item[0];
            let value = &item[2..];

            match key {
                b'r' => {
                    nonce = buf.slice_ref(value);
                }

                b'i' => {
                    iterations = atoi::atoi(value).unwrap_or(4096);
                }

                b's' => {
                    // FIXME: raise proper protocol errors
                    salt = base64::decode(value).unwrap();
                }

                _ => {}
            }
        }

        Ok(Self {
            iterations,
            salt: salt.into_boxed_slice(),

            // FIXME: raise proper protocol errors
            nonce: ByteString::try_from(nonce).unwrap(),
            message: ByteString::try_from(buf).unwrap(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct AuthenticationSaslFinal {
    pub(crate) verifier: Box<[u8]>,
}

impl Deserialize<'_> for AuthenticationSaslFinal {
    fn deserialize_with(buf: Bytes, _: ()) -> Result<Self> {
        let mut verifier = Vec::new();

        for item in buf.split(|b| *b == b',') {
            let key = item[0];
            let value = &item[2..];

            if let b'v' = key {
                // FIXME: raise proper protocol errors
                verifier = base64::decode(value).unwrap();
            }
        }

        Ok(Self { verifier: verifier.into_boxed_slice() })
    }
}
