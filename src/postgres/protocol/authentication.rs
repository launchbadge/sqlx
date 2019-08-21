use super::Decode;

#[derive(Debug)]
pub enum Authentication {
    /// Authentication was successful.
    Ok,

    /// Kerberos V5 authentication is required.
    KerberosV5,

    /// A clear-text password is required.
    CleartextPassword,

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
    // FIXME: authentication mechanisms
    Sasl,

    /// This message contains a SASL challenge.
    SaslContinue { data: Box<[u8]> },

    /// SASL authentication has completed.
    SaslFinal { data: Box<[u8]> },
}

impl Decode for Authentication {
    fn decode(src: &[u8]) -> Self {
        match src[0] {
            0 => Authentication::Ok,
            2 => Authentication::KerberosV5,
            3 => Authentication::CleartextPassword,

            5 => {
                let mut salt = [0_u8; 4];
                salt.copy_from_slice(&src[1..5]);

                Authentication::Md5Password { salt }
            }

            6 => Authentication::ScmCredential,
            7 => Authentication::Gss,
            9 => Authentication::Sspi,

            token => unimplemented!("decode not implemented for token: {}", token),
        }
    }
}
