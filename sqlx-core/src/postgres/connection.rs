use crate::{
    io::{Buf, BufStream},
    postgres::{
        protocol::{self, Decode, Encode, Message},
    },
};
use async_std::net::TcpStream;
use byteorder::NetworkEndian;
use std::{
    io,
    net::{Shutdown, SocketAddr},
};
use crate::postgres::query::PostgresQueryParameters;
use crate::postgres::error::PostgresDatabaseError;

pub struct Postgres {
    stream: BufStream<TcpStream>,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,
}

// [x] 52.2.1. Start-up
// [ ] 52.2.2. Simple Query
// [ ] 52.2.3. Extended Query
// [ ] 52.2.4. Function Call
// [ ] 52.2.5. COPY Operations
// [ ] 52.2.6. Asynchronous Operations
// [ ] 52.2.7. Canceling Requests in Progress
// [x] 52.2.8. Termination
// [ ] 52.2.9. SSL Session Encryption
// [ ] 52.2.10. GSSAPI Session Encryption

impl Postgres {
    pub(super) async fn new(address: SocketAddr) -> crate::Result<Self> {
        let stream = TcpStream::connect(&address).await?;

        Ok(Self {
            stream: BufStream::new(stream),
            process_id: 0,
            secret_key: 0,
        })
    }

    // https://www.postgresql.org/docs/devel/protocol-flow.html#id-1.10.5.7.3
    pub(super) async fn startup(
        &mut self,
        username: &str,
        password: &str,
        database: &str,
    ) -> crate::Result<()> {
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
            println!("recv!?");
            match message {
                Message::Authentication(auth) => {
                    match *auth {
                        protocol::Authentication::Ok => {
                            println!("no auth?");
                            // Do nothing. No password is needed to continue.
                        }

                        protocol::Authentication::CleartextPassword => {
                            protocol::PasswordMessage::Cleartext(password)
                                .encode(self.stream.buffer_mut());

                            self.stream.flush().await?;
                        }

                        protocol::Authentication::Md5Password { salt } => {
                            protocol::PasswordMessage::Md5 {
                                password,
                                user: username,
                                salt,
                            }
                            .encode(self.stream.buffer_mut());

                            self.stream.flush().await?;
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
                    self.process_id = body.process_id();
                    self.secret_key = body.secret_key();
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

        println!("done");

        Ok(())
    }

    // https://www.postgresql.org/docs/devel/protocol-flow.html#id-1.10.5.7.10
    pub(super) async fn terminate(mut self) -> crate::Result<()> {
        protocol::Terminate.encode(self.stream.buffer_mut());

        self.stream.flush().await?;
        self.stream.stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    pub(super) fn parse(&mut self, statement: &str, query: &str, params: &PostgresQueryParameters) {
        protocol::Parse {
            statement,
            query,
            param_types: &*params.types,
        }
        .encode(self.stream.buffer_mut());
    }

    pub(super) fn describe(&mut self, statement: &str) {
        protocol::Describe {
            kind: protocol::DescribeKind::PreparedStatement,
            name: statement,
        }
        .encode(self.stream.buffer_mut())
    }

    pub(super) fn bind(&mut self, portal: &str, statement: &str, params: &PostgresQueryParameters) {
        protocol::Bind {
            portal,
            statement,
            formats: &[1], // [BINARY]
            // TODO: Early error if there is more than i16
            values_len: params.types.len() as i16,
            values: &*params.buf,
            result_formats: &[1], // [BINARY]
        }
        .encode(self.stream.buffer_mut());
    }

    pub(super) fn execute(&mut self, portal: &str, limit: i32) {
        protocol::Execute { portal, limit }.encode(self.stream.buffer_mut());
    }

    pub(super) async fn sync(&mut self) -> crate::Result<()> {
        protocol::Sync.encode(self.stream.buffer_mut());

        self.stream.flush().await?;

        Ok(())
    }

    pub(super) async fn step(&mut self) -> crate::Result<Option<Step>> {
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
                    return Ok(Some(Step::Row(body)));
                }

                Message::ReadyForQuery(_) => {
                    return Ok(None);
                }

                Message::ParameterDescription(desc) => {
                    return Ok(Some(Step::ParamDesc(desc)));
                }

                Message::RowDescription(desc) => {
                    return Ok(Some(Step::RowDesc(desc)));
                }

                message => {
                    return Err(protocol_err!("received unexpected message: {:?}", message).into());
                }
            }
        }

        // Connection was (unexpectedly) closed
        Err(io::Error::from(io::ErrorKind::UnexpectedEof).into())
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
}

#[derive(Debug)]
pub(super) enum Step {
    Command(u64),
    Row(protocol::DataRow),
    ParamDesc(Box<protocol::ParameterDescription>),
    RowDesc(Box<protocol::RowDescription>),
}
