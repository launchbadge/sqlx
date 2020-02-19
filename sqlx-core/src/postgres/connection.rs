use std::convert::TryInto;

use byteorder::NetworkEndian;
use futures_core::future::BoxFuture;
use std::net::Shutdown;

use crate::cache::StatementCache;
use crate::connection::{Connect, Connection};
use crate::io::{Buf, BufStream, MaybeTlsStream};
use crate::postgres::protocol::{self, Authentication, Decode, Encode, Message, StatementId};
use crate::postgres::{sasl, PgError};
use crate::url::Url;
use crate::{Postgres, Result};

/// An asynchronous connection to a [Postgres][super::Postgres] database.
///
/// The connection string expected by [Connect::connect] should be a PostgreSQL connection
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
                                &url.password().unwrap_or_default(),
                            )
                            .encode(self.stream.buffer_mut());

                            self.stream.flush().await?;
                        }

                        protocol::Authentication::Md5Password { salt } => {
                            protocol::PasswordMessage::Md5 {
                                password: &url.password().unwrap_or_default(),
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
                                sasl::authenticate(
                                    self,
                                    username,
                                    &url.password().unwrap_or_default(),
                                )
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
    pub(super) async fn establish(url: Result<Url>) -> Result<Self> {
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

        self_.stream.clear_bufs();

        self_.startup(&url).await?;

        Ok(self_)
    }
}

impl Connect for PgConnection {
    fn connect<T>(url: T) -> BoxFuture<'static, Result<PgConnection>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(PgConnection::establish(url.try_into()))
    }
}

impl Connection for PgConnection {
    type Database = Postgres;

    fn close(self) -> BoxFuture<'static, Result<()>> {
        Box::pin(self.terminate())
    }
}
