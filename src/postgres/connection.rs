use super::{
    protocol::{self, Decode, Encode, Message, Terminate},
    Postgres, PostgresDatabaseError, PostgresQueryParameters, PostgresRow,
};
use crate::{
    connection::RawConnection,
    error::Error,
    io::{Buf, BufStream},
    query::QueryParameters,
    url::Url,
};
use byteorder::NetworkEndian;
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::{
    io,
    net::{IpAddr, Shutdown, SocketAddr},
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct PostgresRawConnection {
    stream: BufStream<TcpStream>,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,

    // Statement ID counter
    next_statement_id: AtomicU64,

    // Portal ID counter
    next_portal_id: AtomicU64,
}

impl PostgresRawConnection {
    async fn establish(url: &str) -> crate::Result<Self> {
        let url = Url::parse(url)?;
        let addr = url.resolve(5432);
        let stream = TcpStream::connect(&addr).await?;

        let mut conn = Self {
            stream: BufStream::new(stream),
            process_id: 0,
            secret_key: 0,
            next_statement_id: AtomicU64::new(0),
            next_portal_id: AtomicU64::new(0),
        };

        let user = url.username();
        let password = url.password().unwrap_or("");
        let database = url.database();

        // See this doc for more runtime parameters
        // https://www.postgresql.org/docs/12/runtime-config-client.html
        let params = &[
            // FIXME: ConnectOptions user and database need to be required parameters and error
            //        before they get here
            ("user", user),
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

        conn.write(protocol::StartupMessage { params });
        conn.stream.flush().await?;

        while let Some(message) = conn.receive().await? {
            match message {
                Message::Authentication(auth) => {
                    match *auth {
                        protocol::Authentication::Ok => {
                            // Do nothing. No password is needed to continue.
                        }

                        protocol::Authentication::CleartextPassword => {
                            conn.write(protocol::PasswordMessage::Cleartext(password));

                            conn.stream.flush().await?;
                        }

                        protocol::Authentication::Md5Password { salt } => {
                            conn.write(protocol::PasswordMessage::Md5 {
                                password,
                                user,
                                salt,
                            });

                            conn.stream.flush().await?;
                        }

                        auth => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("requires unimplemented authentication method: {:?}", auth),
                            )
                            .into());
                        }
                    }
                }

                Message::BackendKeyData(body) => {
                    conn.process_id = body.process_id();
                    conn.secret_key = body.secret_key();
                }

                Message::ReadyForQuery(_) => {
                    break;
                }

                message => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("received unexpected message: {:?}", message),
                    )
                    .into());
                }
            }
        }

        Ok(conn)
    }

    async fn close(&mut self) -> crate::Result<()> {
        self.write(Terminate);
        self.stream.flush().await?;
        self.stream.close().await?;

        Ok(())
    }

    // Wait and return the next message to be received from Postgres.
    async fn receive(&mut self) -> crate::Result<Option<Message>> {
        // Before we start the receive loop
        // Flush any pending data from the send buffer
        self.stream.flush().await?;

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

                id => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("received unexpected message id: {:?}", id),
                    )
                    .into());
                }
            };

            self.stream.consume(len);

            match message {
                Message::ParameterStatus(_body) => {
                    // TODO: not sure what to do with these yet
                }

                Message::Response(body) => {
                    if body.severity().is_error() {
                        // This is an error, stop the world and bubble as an error
                        return Err(PostgresDatabaseError(body).into());
                    } else {
                        // This is a _warning_
                        // TODO: Do we *want* to do anything with these
                    }
                }

                message => {
                    return Ok(Some(message));
                }
            }
        }
    }

    pub(super) fn write(&mut self, message: impl Encode) {
        message.encode(self.stream.buffer_mut());
    }

    fn execute(&mut self, query: &str, params: PostgresQueryParameters, limit: i32) {
        self.write(protocol::Parse {
            portal: "",
            query,
            param_types: &*params.types,
        });

        self.write(protocol::Bind {
            portal: "",
            statement: "",
            formats: &[1], // [BINARY]
            // TODO: Early error if there is more than i16
            values_len: params.types.len() as i16,
            values: &*params.buf,
            result_formats: &[1], // [BINARY]
        });

        // TODO: Make limit be 1 for fetch_optional
        self.write(protocol::Execute { portal: "", limit });

        self.write(protocol::Sync);
    }

    // Ask for the next Row in the stream
    async fn step(&mut self) -> crate::Result<Option<Step>> {
        while let Some(message) = self.receive().await? {
            match message {
                Message::BindComplete
                | Message::ParseComplete
                | Message::PortalSuspended
                | Message::CloseComplete => {}

                Message::CommandComplete(body) => {
                    return Ok(Some(Step::Command(body.affected_rows())));
                }

                Message::DataRow(body) => {
                    return Ok(Some(Step::Row(PostgresRow(body))));
                }

                Message::ReadyForQuery(_) => {
                    return Ok(None);
                }

                message => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("received unexpected message: {:?}", message),
                    )
                    .into());
                }
            }
        }

        // Connection was (unexpectedly) closed
        Err(io::Error::from(io::ErrorKind::UnexpectedEof).into())
    }

    // TODO: Remove usage of fmt!
    fn next_portal_id(&self) -> String {
        format!(
            "__sqlx_portal_{}",
            self.next_portal_id.fetch_add(1, Ordering::AcqRel)
        )
    }

    // TODO: Remove usage of fmt!
    fn next_statement_id(&self) -> String {
        format!(
            "__sqlx_statement_{}",
            self.next_statement_id.fetch_add(1, Ordering::AcqRel)
        )
    }
}

enum Step {
    Command(u64),
    Row(PostgresRow),
}

impl RawConnection for PostgresRawConnection {
    type Backend = Postgres;

    #[inline]
    fn establish(url: &str) -> BoxFuture<crate::Result<Self>> {
        Box::pin(Self::establish(url))
    }

    #[inline]
    fn close<'c>(&'c mut self) -> BoxFuture<'c, crate::Result<()>> {
        Box::pin(self.close())
    }

    fn execute<'c>(
        &'c mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'c, crate::Result<u64>> {
        self.execute(query, params, 1);

        Box::pin(async move {
            let mut affected = 0;

            while let Some(step) = self.step().await? {
                if let Step::Command(cnt) = step {
                    affected = cnt;
                }
            }

            Ok(affected)
        })
    }

    fn fetch<'c>(
        &'c mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxStream<'c, crate::Result<PostgresRow>> {
        self.execute(query, params, 0);

        Box::pin(async_stream::try_stream! {
            while let Some(step) = self.step().await? {
                if let Step::Row(row) = step {
                    yield row;
                }
            }
        })
    }

    fn fetch_optional<'c>(
        &'c mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'c, crate::Result<Option<PostgresRow>>> {
        self.execute(query, params, 1);

        Box::pin(async move {
            let mut row: Option<PostgresRow> = None;

            while let Some(step) = self.step().await? {
                if let Step::Row(r) = step {
                    // This should only ever execute once because we used the
                    // protocol-level limit
                    debug_assert!(row.is_none());
                    row = Some(r);
                }
            }

            Ok(row)
        })
    }
}
