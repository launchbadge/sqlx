use std::convert::TryInto;

use async_std::net::{Shutdown, TcpStream};
use byteorder::NetworkEndian;
use futures_core::future::BoxFuture;

use crate::cache::StatementCache;
use crate::connection::Connection;
use crate::io::{Buf, BufStream};
use crate::postgres::protocol::{self, Decode, Encode, Message, StatementId, SaslResponse, SaslInitialResponse, hi, Authentication};
use crate::postgres::PgError;
use crate::url::Url;
use sha2::{Sha256, Digest};
use hmac::{Mac, Hmac};
use crate::Result;
use rand::Rng;

/// An asynchronous connection to a [Postgres] database.
///
/// The connection string expected by [Connection::open] should be a PostgreSQL connection
/// string, as documented at
/// <https://www.postgresql.org/docs/12/libpq-connect.html#LIBPQ-CONNSTRING>
pub struct PgConnection {
    pub(super) stream: BufStream<TcpStream>,

    // Map of query to statement id
    pub(super) statement_cache: StatementCache<StatementId>,

    // Next statement id
    pub(super) next_statement_id: u32,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,

    // Is there a query in progress; are we ready to continue
    pub(super) ready: bool,
}

impl PgConnection {
    // https://www.postgresql.org/docs/12/protocol-flow.html#id-1.10.5.7.3
    async fn startup(&mut self, url: Url) -> Result<()> {
        // Defaults to postgres@.../postgres
        let username = url.username().unwrap_or("postgres");
        let database = url.database().unwrap_or("postgres");

        // See this doc for more runtime parameters
        // https://www.postgresql.org/docs/12/runtime-config-client.html
        let params = &[
            ("user", username),
            ("database", database),
            // Sets the display format for date and time values,
            // as well as the rules for interpreting ambiguous date input values.
            ("DateStyle", "ISO, MDY"),
            // Sets the display format for interval values.
            ("IntervalStyle", "iso_8601"),
            // Sets the time zone for displaying and interpreting time stamps.
            ("TimeZone", "UTC"),
            // Adjust postgres to return percise values for floats
            // NOTE: This is default in postgres 12+
            ("extra_float_digits", "3"),
            // Sets the client-side encoding (character set).
            ("client_encoding", "UTF-8"),
        ];

        protocol::StartupMessage { params }.encode(self.stream.buffer_mut());
        self.stream.flush().await?;

        while let Some(message) = self.receive().await? {
            match message {
                Message::Authentication(auth) => {
                    match *auth {
                        protocol::Authentication::Ok => {
                            // Do nothing. No password is needed to continue.
                        }

                        protocol::Authentication::ClearTextPassword => {
                            protocol::PasswordMessage::ClearText(
                                url.password().unwrap_or_default(),
                            )
                            .encode(self.stream.buffer_mut());

                            self.stream.flush().await?;
                        }

                        protocol::Authentication::Md5Password { salt } => {
                            protocol::PasswordMessage::Md5 {
                                password: url.password().unwrap_or_default(),
                                user: username,
                                salt,
                            }
                            .encode(self.stream.buffer_mut());

                            self.stream.flush().await?;
                        }

                        protocol::Authentication::Sasl { mechanisms } => {
                            match mechanisms.get(0).map(|m| &**m) {
                                Some("SCRAM-SHA-256") => {
                                    sasl_auth(
                                        self,
                                        username,
                                        url.password().unwrap_or_default(),
                                    )
                                    .await?;
                                }

                                _ => return Err(protocol_err!(
                                    "Expected mechanisms SCRAM-SHA-256, but received {:?}",
                                    mechanisms
                                ).into()),
                            }
                        }

                        auth => {
                            return Err(protocol_err!(
                                "requires unimplemented authentication method: {:?}",
                                auth
                            )
                            .into());
                        }
                    }
                }

                Message::BackendKeyData(body) => {
                    self.process_id = body.process_id;
                    self.secret_key = body.secret_key;
                }

                Message::ReadyForQuery(_) => {
                    // Connection fully established and ready to receive queries.
                    break;
                }

                message => {
                    return Err(protocol_err!("received unexpected message: {:?}", message).into());
                }
            }
        }

        Ok(())
    }

    // https://www.postgresql.org/docs/devel/protocol-flow.html#id-1.10.5.7.10
    async fn terminate(mut self) -> Result<()> {
        protocol::Terminate.encode(self.stream.buffer_mut());

        self.stream.flush().await?;
        self.stream.stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    // Wait and return the next message to be received from Postgres.
    pub(super) async fn receive(&mut self) -> Result<Option<Message>> {
        loop {
            // Read the message header (id + len)
            let mut header = ret_if_none!(self.stream.peek(5).await?);

            let id = header.get_u8()?;
            let len = (header.get_u32::<NetworkEndian>()? - 4) as usize;

            // Read the message body
            self.stream.consume(5);
            let body = ret_if_none!(self.stream.peek(len).await?);

            let message = match id {
                b'N' | b'E' => Message::Response(Box::new(protocol::Response::decode(body)?)),
                b'D' => Message::DataRow(protocol::DataRow::decode(body)?),
                b'S' => {
                    Message::ParameterStatus(Box::new(protocol::ParameterStatus::decode(body)?))
                }
                b'Z' => Message::ReadyForQuery(protocol::ReadyForQuery::decode(body)?),
                b'R' => Message::Authentication(Box::new(protocol::Authentication::decode(body)?)),
                b'K' => Message::BackendKeyData(protocol::BackendKeyData::decode(body)?),
                b'C' => Message::CommandComplete(protocol::CommandComplete::decode(body)?),
                b'A' => Message::NotificationResponse(Box::new(
                    protocol::NotificationResponse::decode(body)?,
                )),
                b'1' => Message::ParseComplete,
                b'2' => Message::BindComplete,
                b'3' => Message::CloseComplete,
                b'n' => Message::NoData,
                b's' => Message::PortalSuspended,
                b't' => Message::ParameterDescription(Box::new(
                    protocol::ParameterDescription::decode(body)?,
                )),
                b'T' => Message::RowDescription(Box::new(protocol::RowDescription::decode(body)?)),

                id => {
                    return Err(protocol_err!("received unknown message id: {:?}", id).into());
                }
            };

            self.stream.consume(len);

            match message {
                Message::ParameterStatus(_body) => {
                    // TODO: not sure what to do with these yet
                }

                Message::Response(body) => {
                    if body.severity.is_error() {
                        // This is an error, stop the world and bubble as an error
                        return Err(PgError(body).into());
                    } else {
                        // This is a _warning_
                        // TODO: Log the warning
                    }
                }

                message => {
                    return Ok(Some(message));
                }
            }
        }
    }
}

impl PgConnection {
    pub(super) async fn open(url: Result<Url>) -> Result<Self> {
        let url = url?;
        let stream = TcpStream::connect((url.host(), url.port(5432))).await?;
        let mut self_ = Self {
            stream: BufStream::new(stream),
            process_id: 0,
            secret_key: 0,
            // Important to start at 1 as 0 means "unnamed" in our protocol
            next_statement_id: 1,
            statement_cache: StatementCache::new(),
            ready: true,
        };

        self_.startup(url).await?;

        Ok(self_)
    }
}

impl Connection for PgConnection {
    fn open<T>(url: T) -> BoxFuture<'static, Result<Self>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(PgConnection::open(url.try_into()))
    }

    fn close(self) -> BoxFuture<'static, Result<()>> {
        Box::pin(self.terminate())
    }
}

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
async fn sasl_auth<T: AsRef<str>>(
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

    SaslInitialResponse(&client_first_message)
    .encode(conn.stream.buffer_mut());
    conn.stream.flush().await?;

    let server_first_message = conn.receive().await?;

    if let Some(Message::Authentication(auth)) = server_first_message {
        if let Authentication::SaslContinue(sasl) = *auth {
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

            SaslResponse(&client_final_message)
            .encode(conn.stream.buffer_mut());
            conn.stream.flush().await?;
            let _server_final_response = conn.receive().await?;

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
