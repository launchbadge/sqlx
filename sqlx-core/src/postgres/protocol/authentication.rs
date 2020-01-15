use crate::io::Buf;
use crate::postgres::protocol::Decode;
use byteorder::NetworkEndian;
use std::str;

#[derive(Debug)]
pub enum Authentication {
    /// Authentication was successful.
    Ok,

    /// Kerberos V5 authentication is required.
    KerberosV5,

    /// A clear-text password is required.
    ClearTextPassword,

    /// An MD5-encrypted password is required.
    Md5Password { salt: [u8; 4] },

    /// An SCM credentials message is required.
    ScmCredential,

    /// GSSAPI authentication is required.
    Gss,

    /// SSPI authentication is required.
    Sspi,

    /// This message contains GSSAPI or SSPI data.
    GssContinue { data: Box<[u8]> },

    /// SASL authentication is required.
    ///
    /// The message body is a list of SASL authentication mechanisms,
    /// in the server's order of preference.
    Sasl { mechanisms: Box<[Box<str>]> },

    /// This message contains a SASL challenge.
    SaslContinue(SaslContinue),

    /// SASL authentication has completed.
    SaslFinal { data: Box<[u8]> },
}

#[derive(Debug)]
pub struct SaslContinue {
    pub salt: Vec<u8>,
    pub iter_count: u32,
    pub nonce: Vec<u8>,
    pub data: String,
}

impl Decode for Authentication {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        Ok(match buf.get_u32::<NetworkEndian>()? {
            0 => Authentication::Ok,

            2 => Authentication::KerberosV5,

            3 => Authentication::ClearTextPassword,

            5 => {
                let mut salt = [0_u8; 4];
                salt.copy_from_slice(&buf);

                Authentication::Md5Password { salt }
            }

            6 => Authentication::ScmCredential,

            7 => Authentication::Gss,

            8 => {
                let mut data = Vec::with_capacity(buf.len());
                data.extend_from_slice(buf);

                Authentication::GssContinue {
                    data: data.into_boxed_slice(),
                }
            }

            9 => Authentication::Sspi,

            10 => {
                let mut mechanisms = Vec::new();

                while buf[0] != 0 {
                    mechanisms.push(buf.get_str_nul()?.into());
                }

                Authentication::Sasl {
                    mechanisms: mechanisms.into_boxed_slice(),
                }
            }

            11 => {
                let mut salt: Vec<u8> = Vec::new();
                let mut nonce: Vec<u8> = Vec::new();
                let mut iter_count: u32 = 0;

                let key_value: Vec<(char, &[u8])> = buf
                    .split(|byte| *byte == b',')
                    .map(|s| {
                        let (key, value) = s.split_at(1);
                        let value = value.split_at(1).1;

                        (key[0] as char, value)
                    })
                    .collect();

                for (key, value) in key_value.iter() {
                    match key {
                        's' => salt = value.to_vec(),
                        'r' => nonce = value.to_vec(),
                        'i' => {
                            let s = str::from_utf8(&value).map_err(|_| {
                                protocol_err!(
                                    "iteration count in sasl response was not a valid utf8 string"
                                )
                            })?;
                            iter_count = u32::from_str_radix(&s, 10).unwrap_or(0);
                        }

                        _ => {}
                    }
                }

                Authentication::SaslContinue(SaslContinue {
                    salt: base64::decode(&salt).map_err(|_| {
                        protocol_err!("salt value response from postgres was not base64 encoded")
                    })?,
                    nonce,
                    iter_count,
                    data: str::from_utf8(buf)
                        .map_err(|_| {
                            protocol_err!("SaslContinue response was not a valid utf8 string")
                        })?
                        .to_string(),
                })
            }

            12 => {
                let mut data = Vec::with_capacity(buf.len());
                data.extend_from_slice(buf);

                Authentication::SaslFinal {
                    data: data.into_boxed_slice(),
                }
            }

            id => {
                return Err(protocol_err!("unknown authentication response: {}", id).into());
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Authentication, Decode};
    use matches::assert_matches;

    const AUTH_OK: &[u8] = b"\0\0\0\0";
    const AUTH_MD5: &[u8] = b"\0\0\0\x05\x93\x189\x98";

    #[test]
    fn it_decodes_auth_ok() {
        let m = Authentication::decode(AUTH_OK).unwrap();

        assert_matches!(m, Authentication::Ok);
    }

    #[test]
    fn it_decodes_auth_md5_password() {
        let m = Authentication::decode(AUTH_MD5).unwrap();

        assert_matches!(
            m,
            Authentication::Md5Password {
                salt: [147, 24, 57, 152]
            }
        );
    }
}
