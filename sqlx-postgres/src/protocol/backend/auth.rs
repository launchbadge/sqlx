use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::protocol::backend::{
    AuthenticationSasl, AuthenticationSaslContinue, AuthenticationSaslFinal,
};
use crate::PgClientError;

#[derive(Debug)]
pub(crate) enum Authentication {
    /// The authentication exchange is successfully completed.
    Ok,

    /// The frontend must now send a PasswordMessage containing the
    /// password in clear-text form.
    CleartextPassword,

    /// The frontend must now send a PasswordMessage containing the
    /// password (with user name) encrypted via MD5.
    Md5Password(AuthenticationMd5Password),

    /// The frontend must now initiate a SASL negotiation,
    /// using one of the SASL mechanisms listed in the message.
    Sasl(AuthenticationSasl),

    /// This message contains challenge data from the previous step of
    /// SASL negotiation.
    SaslContinue(AuthenticationSaslContinue),

    /// SASL authentication has completed with additional mechanism-specific
    /// data for the client.
    SaslFinal(AuthenticationSaslFinal),
}

impl Deserialize<'_> for Authentication {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        match buf.get_u32() {
            0 => Ok(Self::Ok),
            3 => Ok(Self::CleartextPassword),

            5 => {
                let mut salt = [0_u8; 4];
                buf.copy_to_slice(&mut salt);

                Ok(Self::Md5Password(AuthenticationMd5Password { salt }))
            }

            10 => AuthenticationSasl::deserialize(buf).map(Self::Sasl),
            11 => AuthenticationSaslContinue::deserialize(buf).map(Self::SaslContinue),
            12 => AuthenticationSaslFinal::deserialize(buf).map(Self::SaslFinal),

            ty => Err(PgClientError::UnknownAuthenticationMethod(ty).into()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AuthenticationMd5Password {
    pub(crate) salt: [u8; 4],
}
