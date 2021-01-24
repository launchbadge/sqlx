use hmac::{Hmac, Mac, NewMac};
use rand::Rng;
use sha2::digest::Digest;
use sha2::Sha256;
use sqlx_core::Error;
use sqlx_core::Result;

use crate::protocol::{
    Authentication, AuthenticationSasl, MessageType, SaslInitialResponse, SaslResponse,
};

pub(super) const GS2_HEADER: &str = "n,,";
pub(super) const CHANNEL_ATTR: &str = "c";
pub(super) const USERNAME_ATTR: &str = "n";
pub(super) const CLIENT_PROOF_ATTR: &str = "p";
pub(super) const NONCE_ATTR: &str = "r";

macro_rules! sasl_authenticate {
    (@blocking @packet $self:ident) => {
        $self.read_packet()?;
    };

    (@packet $self:ident) => {
        $self.read_packet_async().await?;
    };

    ($(@$blocking:ident)? $self:ident, $options:ident, $data:ident) => {{
        let mut has_sasl = false;
        let mut has_sasl_plus = false;
        let mut unknown = Vec::new();

        for mechanism in $data.mechanisms() {
            match mechanism {
                "SCRAM-SHA-256" => {
                    has_sasl = true;
                }

                "SCRAM-SHA-256-PLUS" => {
                    has_sasl_plus = true;
                }

                _ => {
                    unknown.push(mechanism.to_owned());
                }
            }
        }

        if !has_sasl_plus && !has_sasl {
            return Err(Error::connect(crate::PostgresDatabaseError::protocol(format!(
                "unsupported SASL authentication mechanisms: {}",
                unknown.join(", ")
            ))));
        }

        // channel-binding = "c=" base64
        let channel_binding = format!("{}={}", crate::connection::sasl::CHANNEL_ATTR, base64::encode(crate::connection::sasl::GS2_HEADER));

        // "n=" saslname ;; Usernames are prepared using SASLprep.
        let username = format!("{}={}", crate::connection::sasl::USERNAME_ATTR, $options.get_username().unwrap_or_default());
        let username = match stringprep::saslprep(&username) {
            Ok(v) => v,
            Err(err) => {
                return Err(Error::connect(crate::PostgresDatabaseError::protocol(format!(
                                "failed to sasl prep the username: {:?}", err
                ))));
            }
        };

        // nonce = "r=" c-nonce [s-nonce] ;; Second part provided by server.
        let nonce = crate::connection::sasl::gen_nonce();

        // client-first-message-bare = [reserved-mext ","] username "," nonce ["," extensions]
        let client_first_message_bare =
            format!("{username},{nonce}", username = username, nonce = nonce);

        let client_first_message = format!(
            "{gs2_header}{client_first_message_bare}",
            gs2_header = crate::connection::sasl::GS2_HEADER,
            client_first_message_bare = client_first_message_bare
        );

        $self.write_packet(&crate::protocol::SaslInitialResponse { response: &client_first_message, plus: false })?;

        let message: Message = sasl_authenticate!($(@$blocking)? @packet $self);
        let cont = match message.r#type {
            MessageType::Authentication => {
                match message.decode()? {
                    Authentication::SaslContinue(data) => data,

                    auth => {
                        return Err(Error::connect(PostgresDatabaseError::protocol(format!(
                            "expected SASLContinue but received {:?}", auth
                        ))));
                    }
                }
            }

            _ => {
                todo!()
            }
        };

        // SaltedPassword := Hi(Normalize(password), salt, i)
        let salted_password =
            crate::connection::sasl::hi($options.get_password().unwrap_or_default(), &cont.salt, cont.iterations)?;

        // ClientKey := HMAC(SaltedPassword, "Client Key")
        let mut mac = Hmac::<Sha256>::new_varkey(&salted_password)
            .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;
        mac.update(b"Client Key");

        let client_key = mac.finalize().into_bytes();

        // StoredKey := H(ClientKey)
        let stored_key = Sha256::digest(&client_key);

        // client-final-message-without-proof
        let client_final_message_wo_proof = format!(
            "{channel_binding},r={nonce}",
            channel_binding = channel_binding,
            nonce = &cont.nonce
        );

        // AuthMessage := client-first-message-bare + "," + server-first-message + "," + client-final-message-without-proof
        let auth_message = format!(
            "{client_first_message_bare},{server_first_message},{client_final_message_wo_proof}",
            client_first_message_bare = client_first_message_bare,
            server_first_message = cont.message,
            client_final_message_wo_proof = client_final_message_wo_proof
        );

        // ClientSignature := HMAC(StoredKey, AuthMessage)
        let mut mac = Hmac::<Sha256>::new_varkey(&stored_key)
            .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;
        mac.update(&auth_message.as_bytes());

        let client_signature = mac.finalize().into_bytes();

        // ClientProof := ClientKey XOR ClientSignature
        let client_proof: Vec<u8> =
            client_key.iter().zip(client_signature.iter()).map(|(&a, &b)| a ^ b).collect();

        // ServerKey := HMAC(SaltedPassword, "Server Key")
        let mut mac = Hmac::<Sha256>::new_varkey(&salted_password)
            .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;
        mac.update(b"Server Key");

        let server_key = mac.finalize().into_bytes();

        // ServerSignature := HMAC(ServerKey, AuthMessage)
        let mut mac = Hmac::<Sha256>::new_varkey(&server_key)
            .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;
        mac.update(&auth_message.as_bytes());

        // client-final-message = client-final-message-without-proof "," proof
        let client_final_message = format!(
            "{client_final_message_wo_proof},{client_proof_attr}={client_proof}",
            client_final_message_wo_proof = client_final_message_wo_proof,
            client_proof_attr = crate::connection::sasl::CLIENT_PROOF_ATTR,
            client_proof = base64::encode(&client_proof)
        );

        $self.write_packet(&crate::protocol::SaslResponse(&client_final_message))?;

        let message: Message = sasl_authenticate!($(@$blocking)? @packet $self);
        let data = match message.r#type {
            MessageType::Authentication => {
                match message.decode()? {
                    Authentication::SaslFinal(data) => data,

                    auth => {
                        return Err(Error::connect(PostgresDatabaseError::protocol(format!(
                            "expected SASLContinue but received {:?}", auth
                        ))));
                    }
                }
            }

            r#type => {
                return Err(Error::connect(PostgresDatabaseError::protocol(format!(
                    "Expected an authencation message type, found {:?}", r#type
                ))));
            }
        };

        // authentication is only considered valid if this verification passes
        mac.verify(&data.verifier)
            .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;
    }};
}

// nonce is a sequence of random printable bytes
pub(super) fn gen_nonce() -> String {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(64, 128);

    // printable = %x21-2B / %x2D-7E
    // ;; Printable ASCII except ",".
    // ;; Note that any "printable" is also
    // ;; a valid "value".
    let nonce: String = std::iter::repeat(())
        .map(|()| {
            let mut c = rng.gen_range(0x21, 0x7F) as u8;

            while c == 0x2C {
                c = rng.gen_range(0x21, 0x7F) as u8;
            }

            c
        })
        .take(count)
        .map(|c| c as char)
        .collect();

    rng.gen_range(32, 128);
    format!("{}={}", crate::connection::sasl::NONCE_ATTR, nonce)
}

// Hi(str, salt, i):
pub(super) fn hi<'a>(s: &'a str, salt: &'a [u8], iter_count: u32) -> Result<[u8; 32]> {
    let mut mac = Hmac::<Sha256>::new_varkey(s.as_bytes())
        .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;

    mac.update(&salt);
    mac.update(&1u32.to_be_bytes());

    let mut u = mac.finalize().into_bytes();
    let mut hi = u;

    for _ in 1..iter_count {
        let mut mac = Hmac::<Sha256>::new_varkey(s.as_bytes())
            .map_err(|err| Error::connect(crate::PostgresDatabaseError::from(err)))?;
        mac.update(u.as_slice());
        u = mac.finalize().into_bytes();
        hi = hi.iter().zip(u.iter()).map(|(&a, &b)| a ^ b).collect();
    }

    Ok(hi.into())
}
