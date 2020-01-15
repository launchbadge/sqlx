use std::convert::TryInto;

use async_std::net::Shutdown;
use byteorder::NetworkEndian;
use futures_core::future::BoxFuture;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::cache::StatementCache;
use crate::connection::Connection;
use crate::io::{Buf, BufStream, MaybeTlsStream};
use crate::postgres::protocol::{
    self, hi, Authentication, Decode, Encode, Message, SaslInitialResponse, SaslResponse,
    StatementId,
};
use crate::postgres::PgError;
use crate::url::Url;
use crate::Result;

/// An asynchronous connection to a [Postgres] database.
///
/// The connection string expected by [Connection::open] should be a PostgreSQL connection
/// string, as documented at
/// <https://www.postgresql.org/docs/12/libpq-connect.html#LIBPQ-CONNSTRING>
///
/// ### TLS Support (requires `tls` feature)
/// This connection type supports the same `sslmode` query parameter that `libpq` does in
/// connection strings: <https://www.postgresql.org/docs/12/libpq-ssl.html>
///
/// ```text
/// postgresql://<user>[:<password>]@<host>[:<port>]/<database>[?sslmode=<ssl-mode>[&sslcrootcert=<path>]]
/// ```
/// where
/// ```text
/// ssl-mode = disable | allow | prefer | require | verify-ca | verify-full
/// path = percent (URL) encoded path on the local machine
/// ```
///
/// If the `tls` feature is not enabled, `disable`, `allow` and `prefer` are no-ops and `require`,
/// `verify-ca` and `verify-full` are forbidden (attempting to connect with these will return
/// an error).
///
/// If the `tls` feature is enabled, an upgrade to TLS is attempted on every connection by default
/// (equivalent to `sslmode=prefer`). If the server does not support TLS (because it was not
/// started with a valid certificate and key, see <https://www.postgresql.org/docs/12/ssl-tcp.html>)
/// then it falls back to an unsecured connection and logs a warning.
///
/// Add `sslmode=require` to your connection string to emit an error if the TLS upgrade fails.
///
/// If you're running Postgres locally, your connection string might look like this:
/// ```text
/// postgresql://root:password@localhost/my_database?sslmode=require
/// ```
///
/// However, like with `libpq` the server certificate is **not** checked for validity by default.
///
/// Specifying `sslmode=verify-ca` will cause the TLS upgrade to verify the server's SSL
/// certificate against a local CA root certificate; this is not the system root certificate
/// but is instead expected to be specified in one of a few ways:
///
/// * The path to the certificate can be specified by adding the `sslrootcert` query parameter
/// to the connection string. (Remember to percent-encode it!)
///
/// * The path may also be specified via the `PGSSLROOTCERT` environment variable (which
/// should *not* be percent-encoded.)
///
/// * Otherwise, the library will look for the Postgres global root CA certificate in the default
/// location:
///
///     * `$HOME/.postgresql/root.crt` on POSIX systems
///     * `%APPDATA%\postgresql\root.crt` on Windows
///
/// These locations are documented here: <https://www.postgresql.org/docs/12/libpq-ssl.html#LIBQ-SSL-CERTIFICATES>
/// If the root certificate cannot be found by any of these means then the TLS upgrade will fail.
///
/// If `sslmode=verify-full` is specified, in addition to checking the certificate as with
/// `sslmode=verify-ca`, the hostname in the connection string will be verified
/// against the hostname in the server certificate, so they must be the same for the TLS
/// upgrade to succeed.
pub struct PgConnection {
    pub(super) stream: BufStream<MaybeTlsStream>,

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
    #[cfg(feature = "tls")]
    async fn try_ssl(
        &mut self,
        url: &Url,
        invalid_certs: bool,
        invalid_hostnames: bool,
    ) -> crate::Result<bool> {
        use async_native_tls::TlsConnector;

        protocol::SslRequest::encode(self.stream.buffer_mut());

        self.stream.flush().await?;

        match self.stream.peek(1).await? {
            Some(b"N") => return Ok(false),
            Some(b"S") => (),
            Some(other) => {
                return Err(tls_err!("unexpected single-byte response: 0x{:02X}", other[0]).into())
            }
            None => return Err(tls_err!("server unexpectedly closed connection").into()),
        }

        let mut connector = TlsConnector::new()
            .danger_accept_invalid_certs(invalid_certs)
            .danger_accept_invalid_hostnames(invalid_hostnames);

        if !invalid_certs {
            match read_root_certificate(&url).await {
                Ok(cert) => {
                    connector = connector.add_root_certificate(cert);
                }
                Err(e) => log::warn!("failed to read Postgres root certificate: {}", e),
            }
        }

        self.stream.clear_bufs();
        self.stream.stream.upgrade(url, connector).await?;

        Ok(true)
    }

    // https://www.postgresql.org/docs/12/protocol-flow.html#id-1.10.5.7.3
    async fn startup(&mut self, url: &Url) -> Result<()> {
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
                            let mut has_sasl: bool = false;
                            let mut has_sasl_plus: bool = false;

                            for mechanism in &*mechanisms {
                                match &**mechanism {
                                    "SCRAM-SHA-256" => {
                                        has_sasl = true;
                                    }

                                    "SCRAM-SHA-256-PLUS" => {
                                        has_sasl_plus = true;
                                    }

                                    _ => {
                                        log::info!("unsupported auth mechanism: {}", mechanism);
                                    }
                                }
                            }

                            if has_sasl || has_sasl_plus {
                                // TODO: Handle -PLUS differently if we're in a TLS stream
                                sasl_auth(self, username, url.password().unwrap_or_default())
                                    .await?;
                            } else {
                                return Err(protocol_err!(
                                    "unsupported SASL auth mechanisms: {:?}",
                                    mechanisms
                                )
                                .into());
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

        let stream = MaybeTlsStream::connect(&url, 5432).await?;
        let mut self_ = Self {
            stream: BufStream::new(stream),
            process_id: 0,
            secret_key: 0,
            // Important to start at 1 as 0 means "unnamed" in our protocol
            next_statement_id: 1,
            statement_cache: StatementCache::new(),
            ready: true,
        };

        let ssl_mode = url.get_param("sslmode").unwrap_or("prefer".into());

        match &*ssl_mode {
            // TODO: on "allow" retry with TLS if startup fails
            "disable" | "allow" => (),

            #[cfg(feature = "tls")]
            "prefer" => {
                if !self_.try_ssl(&url, true, true).await? {
                    log::warn!("server does not support TLS, falling back to unsecured connection")
                }
            }

            #[cfg(not(feature = "tls"))]
            "prefer" => log::info!("compiled without TLS, skipping upgrade"),

            #[cfg(feature = "tls")]
            "require" | "verify-ca" | "verify-full" => {
                if !self_
                    .try_ssl(
                        &url,
                        ssl_mode == "require", // false for both verify-ca and verify-full
                        ssl_mode != "verify-full", // false for only verify-full
                    )
                    .await?
                {
                    return Err(tls_err!("Postgres server does not support TLS").into());
                }
            }

            #[cfg(not(feature = "tls"))]
            "require" | "verify-ca" | "verify-full" => {
                return Err(tls_err!(
                    "sslmode {:?} unsupported; SQLx was compiled without `tls` feature",
                    ssl_mode
                )
                .into())
            }
            _ => return Err(tls_err!("unknown `sslmode` value: {:?}", ssl_mode).into()),
        }

        self_.startup(&url).await?;

        Ok(self_)
    }
}

impl PgConnection {
    #[deprecated(note = "please use 'connect' instead")]
    pub fn open<T>(url: T) -> BoxFuture<'static, Result<Self>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(PgConnection::open(url.try_into()))
    }
}

impl Connect for PgConnection {
    type Connection = PgConnection;

    fn connect<T>(url: T) -> BoxFuture<'static, Result<PgConnection>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(PgConnection::open(url.try_into()))
    }
}

impl Connection for PgConnection {
    fn close(self) -> BoxFuture<'static, Result<()>> {
        Box::pin(self.terminate())
    }
}

#[cfg(feature = "tls")]
async fn read_root_certificate(url: &Url) -> crate::Result<async_native_tls::Certificate> {
    use std::env;

    let root_cert_path = if let Some(path) = url.get_param("sslrootcert") {
        path.into()
    } else if let Ok(cert_path) = env::var("PGSSLROOTCERT") {
        cert_path
    } else if cfg!(windows) {
        let appdata = env::var("APPDATA").map_err(|_| tls_err!("APPDATA not set"))?;
        format!("{}\\postgresql\\root.crt", appdata)
    } else {
        let home = env::var("HOME").map_err(|_| tls_err!("HOME not set"))?;
        format!("{}/.postgresql/root.crt", home)
    };

    let root_cert = async_std::fs::read(root_cert_path).await?;
    Ok(async_native_tls::Certificate::from_pem(&root_cert)?)
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
async fn sasl_auth<T: AsRef<str>>(conn: &mut PgConnection, username: T, password: T) -> Result<()> {
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

    SaslInitialResponse(&client_first_message).encode(conn.stream.buffer_mut());
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

            SaslResponse(&client_final_message).encode(conn.stream.buffer_mut());
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
