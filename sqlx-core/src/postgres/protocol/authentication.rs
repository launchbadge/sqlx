use crate::io::Buf;
use crate::postgres::database::Postgres;
use byteorder::NetworkEndian;
use std::str;

#[derive(Debug)]
pub(crate) enum Authentication {
    /// The authentication exchange is successfully completed.
    Ok,

    /// The frontend must now take part in a Kerberos V5 authentication dialog (not described
    /// here, part of the Kerberos specification) with the server. If this is successful,
    /// the server responds with an `AuthenticationOk`, otherwise it responds
    /// with an `ErrorResponse`. This is no longer supported.
    KerberosV5,

    /// The frontend must now send a `PasswordMessage` containing the password in clear-text form.
    /// If this is the correct password, the server responds with an `AuthenticationOk`, otherwise it
    /// responds with an `ErrorResponse`.
    CleartextPassword,

    /// The frontend must now send a `PasswordMessage` containing the password (with user name)
    /// encrypted via MD5, then encrypted again using the 4-byte random salt specified in the
    /// `AuthenticationMD5Password` message. If this is the correct password, the server responds
    /// with an `AuthenticationOk`, otherwise it responds with an `ErrorResponse`.
    Md5Password,

    /// This response is only possible for local Unix-domain connections on platforms that support
    /// SCM credential messages. The frontend must issue an SCM credential message and then
    /// send a single data byte.
    ScmCredential,

    /// The frontend must now initiate a GSSAPI negotiation. The frontend will send a
    /// `GSSResponse` message with the first part of the GSSAPI data stream in response to this.
    Gss,

    /// The frontend must now initiate a SSPI negotiation.
    /// The frontend will send a GSSResponse with the first part of the SSPI data stream in
    /// response to this.
    Sspi,

    /// This message contains the response data from the previous step of GSSAPI
    /// or SSPI negotiation.
    GssContinue,

    /// The frontend must now initiate a SASL negotiation, using one of the SASL mechanisms
    /// listed in the message.
    Sasl,

    /// This message contains challenge data from the previous step of SASL negotiation.
    SaslContinue,

    /// SASL authentication has completed with additional mechanism-specific data for the client.
    SaslFinal,
}

impl Authentication {
    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<Postgres, Self> {
        Ok(match buf.get_u32::<NetworkEndian>()? {
            0 => Authentication::Ok,
            2 => Authentication::KerberosV5,
            3 => Authentication::CleartextPassword,
            5 => Authentication::Md5Password,
            6 => Authentication::ScmCredential,
            7 => Authentication::Gss,
            8 => Authentication::GssContinue,
            9 => Authentication::Sspi,
            10 => Authentication::Sasl,
            11 => Authentication::SaslContinue,
            12 => Authentication::SaslFinal,

            type_ => {
                return Err(protocol_err!("unknown authentication message type: {}", type_).into());
            }
        })
    }
}

#[derive(Debug)]
pub(crate) struct AuthenticationMd5 {
    pub(crate) salt: [u8; 4],
}

impl AuthenticationMd5 {
    pub(crate) fn read(buf: &[u8]) -> crate::Result<Postgres, Self> {
        let mut salt = [0_u8; 4];
        salt.copy_from_slice(buf);

        Ok(Self { salt })
    }
}

#[derive(Debug)]
pub(crate) struct AuthenticationSasl {
    pub(crate) mechanisms: Box<[Box<str>]>,
}

impl AuthenticationSasl {
    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<Postgres, Self> {
        let mut mechanisms = Vec::new();

        while buf[0] != 0 {
            mechanisms.push(buf.get_str_nul()?.into());
        }

        Ok(Self {
            mechanisms: mechanisms.into_boxed_slice(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct AuthenticationSaslContinue {
    pub(crate) salt: Vec<u8>,
    pub(crate) iter_count: u32,
    pub(crate) nonce: Vec<u8>,
    pub(crate) data: String,
}

impl AuthenticationSaslContinue {
    pub(crate) fn read(buf: &[u8]) -> crate::Result<Postgres, Self> {
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

        Ok(Self {
            salt: base64::decode(&salt).map_err(|_| {
                protocol_err!("salt value response from postgres was not base64 encoded")
            })?,
            nonce,
            iter_count,
            data: str::from_utf8(buf)
                .map_err(|_| protocol_err!("SaslContinue response was not a valid utf8 string"))?
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Authentication;
    use crate::postgres::protocol::authentication::AuthenticationMd5;
    use matches::assert_matches;

    const AUTH_OK: &[u8] = b"\0\0\0\0";
    const AUTH_MD5: &[u8] = b"\0\0\0\x05\x93\x189\x98";

    #[test]
    fn it_reads_auth_ok() {
        let m = Authentication::read(AUTH_OK).unwrap();

        assert_matches!(m, Authentication::Ok);
    }

    #[test]
    fn it_reads_auth_md5_password() {
        let m = Authentication::read(AUTH_MD5).unwrap();
        let data = AuthenticationMd5::read(&AUTH_MD5[4..]).unwrap();

        assert_matches!(m, Authentication::Md5Password);
        assert_eq!(data.salt, [147, 24, 57, 152]);
    }
}
