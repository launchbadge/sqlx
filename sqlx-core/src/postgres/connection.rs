use std::convert::TryInto;

use async_std::net::{Shutdown, TcpStream};
use byteorder::NetworkEndian;
use futures_core::future::BoxFuture;

use crate::cache::StatementCache;
use crate::connection::Connection;
use crate::io::{Buf, BufStream};
use crate::postgres::protocol::{self, Decode, Encode, Message, StatementId};
use crate::postgres::PgError;
use crate::url::Url;
use std::ops::Deref;

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
    async fn startup(&mut self, url: Url) -> crate::Result<()> {
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
                            let mechanism = (*mechanisms)
                                .get(0)
                                .ok_or(protocol_err!(
                                    "Expected mechanisms SCRAM-SHA-256, but received {:?}",
                                    mechanisms
                                ))?
                                .deref();
                            if "SCRAM-SHA-256" == &*mechanism {
                                protocol::sasl_auth(
                                    self,
                                    username,
                                    url.password().unwrap_or_default(),
                                )
                                .await
                            } else {
                                Err(protocol_err!(
                                    "Expected mechanisms SCRAM-SHA-256, but received {:?}",
                                    mechanisms
                                ))?
                            }?;
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
    async fn terminate(mut self) -> crate::Result<()> {
        protocol::Terminate.encode(self.stream.buffer_mut());

        self.stream.flush().await?;
        self.stream.stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    // Wait and return the next message to be received from Postgres.
    pub(super) async fn receive(&mut self) -> crate::Result<Option<Message>> {
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
    pub(super) async fn open(url: crate::Result<Url>) -> crate::Result<Self> {
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
    fn open<T>(url: T) -> BoxFuture<'static, crate::Result<Self>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(PgConnection::open(url.try_into()))
    }

    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(self.terminate())
    }
}
