use crate::io::BufMut;
use crate::postgres::connection::PgConnection;
use crate::postgres::protocol::authentication::Authentication::SaslContinue;
use crate::postgres::protocol::Encode;
use crate::postgres::protocol::Message;
use crate::Result;
use byteorder::NetworkEndian;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::{Digest, Sha256};

pub struct SaslInitialResponse {
    // pub username: String,
    // pub passord: String,
    pub s: String,
}

impl Encode for SaslInitialResponse {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'p');
        buf.put_u32::<NetworkEndian>(4u32 + self.s.as_str().as_bytes().len() as u32 + 14u32 + 4u32);
        buf.put_str_nul("SCRAM-SHA-256");
        buf.put_u32::<NetworkEndian>(self.s.as_str().as_bytes().len() as u32);
        buf.extend_from_slice(self.s.as_str().as_bytes());
    }
}

pub struct SaslResponse {
    pub s: String,
}

impl Encode for SaslResponse {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(b'p');
        buf.put_u32::<NetworkEndian>(4u32 + self.s.as_str().as_bytes().len() as u32);
        buf.extend_from_slice(self.s.as_str().as_bytes());
    }
}

static GS2_HEADER: &'static str = "n,,";
static CHANNEL_ATTR: &'static str = "c";
static USERNAME_ATTR: &'static str = "n";
static CLIENT_PROOF_ATTR: &'static str = "p";
static NONCE_ATTR: &'static str = "r";

pub fn nonce() -> String {
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

pub async fn sasl_auth<T: AsRef<str>>(
    conn: &mut PgConnection,
    username: T,
    password: T,
) -> Result<()> {
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

    SaslInitialResponse {
        s: client_first_message,
    }
    .encode(conn.stream.buffer_mut());
    conn.stream.flush().await?;

    let server_first_message = conn.receive().await?;

    if let Some(Message::Authentication(auth)) = server_first_message {
        if let SaslContinue(sasl) = *auth {
            let server_first_message = sasl.data;

            // SaltedPassword := Hi(Normalize(password), salt, i)
            let salted_password = hi(password.as_ref(), sasl.salt, sasl.iter_count);

            // ClientKey := HMAC(SaltedPassword, "Client Key")
            let mut mac = Hmac::<Sha256>::new_varkey(&salted_password)
                .expect("HMAC can take key of any size");
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
                .expect("HMAC can take key of any size");
            mac.input(b"Server Key");
            let server_key = mac.result().code();

            // ServerSignature := HMAC(ServerKey, AuthMessage)
            let mut mac =
                Hmac::<Sha256>::new_varkey(&server_key).expect("HMAC can take key of any size");
            mac.input(&auth_message.as_bytes());
            let server_signature = mac.result().code();

            // client-final-message = client-final-message-without-proof "," proof
            let client_final_message = format!(
                "{client_final_message_wo_proof},p={client_proof}",
                client_final_message_wo_proof = client_final_message_wo_proof,
                client_proof = base64::encode(&client_proof)
            );

            SaslResponse {
                s: client_final_message,
            }
            .encode(conn.stream.buffer_mut());
            conn.stream.flush().await?;
            let server_final_response = conn.receive().await?;

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

// Hi(str, salt, i):
pub fn hi<T: AsRef<str>>(s: T, salt: Vec<u8>, iter_count: u32) -> Vec<u8> {
    let mut mac =
        Hmac::<Sha256>::new_varkey(s.as_ref().as_bytes()).expect("HMAC can take key of any size");

    mac.input(&salt);
    mac.input(&1u32.to_be_bytes());

    let mut u = mac.result().code();
    let mut hi = u;

    for _ in 1..iter_count {
        let mut mac = Hmac::<Sha256>::new_varkey(s.as_ref().as_bytes())
            .expect("HMAC can take key of any size");
        mac.input(u.as_slice());
        u = mac.result().code();
        hi = hi.iter().zip(u.iter()).map(|(&a, &b)| a ^ b).collect();
    }

    hi.to_vec()
}
