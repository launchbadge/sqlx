use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::postgres::database::Postgres;
use crate::postgres::protocol::{
    hi, Authentication, AuthenticationSaslContinue, Message, SaslInitialResponse, SaslResponse,
};
use crate::postgres::stream::PgStream;

static GS2_HEADER: &'static str = "n,,";
static CHANNEL_ATTR: &'static str = "c";
static USERNAME_ATTR: &'static str = "n";
static CLIENT_PROOF_ATTR: &'static str = "p";
static NONCE_ATTR: &'static str = "r";

// Nonce generator
// Nonce is a sequence of random printable bytes
fn nonce() -> String {
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
    format!("{}={}", NONCE_ATTR, nonce)
}

// Performs authenticiton using Simple Authentication Security Layer (SASL) which is what
// Postgres uses
pub(super) async fn authenticate<T: AsRef<str>>(
    stream: &mut PgStream,
    username: T,
    password: T,
) -> crate::Result<Postgres, ()> {
    // channel-binding = "c=" base64
    let channel_binding = format!("{}={}", CHANNEL_ATTR, base64::encode(GS2_HEADER));
    // "n=" saslname ;; Usernames are prepared using SASLprep.
    let username = format!("{}={}", USERNAME_ATTR, username.as_ref());
    // nonce = "r=" c-nonce [s-nonce] ;; Second part provided by server.
    let nonce = nonce();
    let client_first_message_bare =
        format!("{username},{nonce}", username = username, nonce = nonce);
    // client-first-message-bare = [reserved-mext ","] username "," nonce ["," extensions]
    let client_first_message = format!(
        "{gs2_header}{client_first_message_bare}",
        gs2_header = GS2_HEADER,
        client_first_message_bare = client_first_message_bare
    );

    stream.write(SaslInitialResponse(&client_first_message));
    stream.flush().await?;

    let server_first_message = stream.receive().await?;

    if let Message::Authentication = server_first_message {
        let auth = Authentication::read(stream.buffer())?;

        if let Authentication::SaslContinue = auth {
            // todo: better way to indicate that we consumed just these 4 bytes?
            let sasl = AuthenticationSaslContinue::read(&stream.buffer()[4..])?;

            let server_first_message = sasl.data;

            // SaltedPassword := Hi(Normalize(password), salt, i)
            let salted_password = hi(password.as_ref(), &sasl.salt, sasl.iter_count)?;

            // ClientKey := HMAC(SaltedPassword, "Client Key")
            let mut mac = Hmac::<Sha256>::new_varkey(&salted_password)
                .map_err(|_| protocol_err!("HMAC can take key of any size"))?;
            mac.input(b"Client Key");
            let client_key = mac.result().code();

            // StoredKey := H(ClientKey)
            let mut hasher = Sha256::new();
            hasher.input(client_key);
            let stored_key = hasher.result();

            // String::from_utf8_lossy should never fail because Postgres requires
            // the nonce to be all printable characters except ','
            let client_final_message_wo_proof = format!(
                "{channel_binding},r={nonce}",
                channel_binding = channel_binding,
                nonce = String::from_utf8_lossy(&sasl.nonce)
            );

            // AuthMessage := client-first-message-bare + "," + server-first-message + "," + client-final-message-without-proof
            let auth_message = format!("{client_first_message_bare},{server_first_message},{client_final_message_wo_proof}",
                                       client_first_message_bare = client_first_message_bare,
                                       server_first_message = server_first_message,
                                       client_final_message_wo_proof = client_final_message_wo_proof);

            // ClientSignature := HMAC(StoredKey, AuthMessage)
            let mut mac =
                Hmac::<Sha256>::new_varkey(&stored_key).expect("HMAC can take key of any size");
            mac.input(&auth_message.as_bytes());
            let client_signature = mac.result().code();

            // ClientProof := ClientKey XOR ClientSignature
            let client_proof: Vec<u8> = client_key
                .iter()
                .zip(client_signature.iter())
                .map(|(&a, &b)| a ^ b)
                .collect();

            // ServerKey := HMAC(SaltedPassword, "Server Key")
            let mut mac = Hmac::<Sha256>::new_varkey(&salted_password)
                .map_err(|_| protocol_err!("HMAC can take key of any size"))?;
            mac.input(b"Server Key");
            let server_key = mac.result().code();

            // ServerSignature := HMAC(ServerKey, AuthMessage)
            let mut mac =
                Hmac::<Sha256>::new_varkey(&server_key).expect("HMAC can take key of any size");
            mac.input(&auth_message.as_bytes());
            let _server_signature = mac.result().code();

            // client-final-message = client-final-message-without-proof "," proof
            let client_final_message = format!(
                "{client_final_message_wo_proof},{client_proof_attr}={client_proof}",
                client_final_message_wo_proof = client_final_message_wo_proof,
                client_proof_attr = CLIENT_PROOF_ATTR,
                client_proof = base64::encode(&client_proof)
            );

            stream.write(SaslResponse(&client_final_message));
            stream.flush().await?;

            let _server_final_response = stream.receive().await?;
            // todo: assert that this was SaslFinal?

            Ok(())
        } else {
            Err(protocol_err!(
                "Expected Authentication::SaslContinue, but received {:?}",
                auth
            ))?
        }
    } else {
        Err(protocol_err!(
            "Expected Message::Authentication, but received {:?}",
            server_first_message
        ))?
    }
}
