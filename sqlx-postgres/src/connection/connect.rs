//! Implements the connection phase.
//!
//! The connection phase (establish) performs these tasks:
//!
//! -   exchange the capabilities of client and server
//! -   setup SSL communication channel if requested
//! -   authenticate the client against the server
//!
//! The server may immediately send an ERR packet and finish the handshake
//! or send a `Handshake`.
//!
//! https://dev.postgres.com/doc/internals/en/connection-phase.html
//!
use hmac::{Hmac, Mac, NewMac};
use sha2::{Digest, Sha256};
use sqlx_core::net::Stream as NetStream;
use sqlx_core::Error;
use sqlx_core::Result;

use crate::protocol::{
    Authentication, BackendKeyData, Message, MessageType, Password, ReadyForQuery,
    SaslInitialResponse, SaslResponse, Startup,
};
use crate::{PostgresConnectOptions, PostgresConnection, PostgresDatabaseError};

macro_rules! connect {
    (@blocking @tcp $options:ident) => {
        NetStream::connect($options.address.as_ref())?;
    };

    (@tcp $options:ident) => {
        NetStream::connect_async($options.address.as_ref()).await?;
    };

    (@blocking @packet $self:ident) => {
        $self.read_packet()?;
    };

    (@packet $self:ident) => {
        $self.read_packet_async().await?;
    };

    ($(@$blocking:ident)? $options:ident) => {{
        // open a network stream to the database server
        let stream = connect!($(@$blocking)? @tcp $options);

        // construct a <PostgresConnection> around the network stream
        // wraps the stream in a <BufStream> to buffer read and write
        let mut self_ = Self::new(stream);

        // To begin a session, a frontend opens a connection to the server
        // and sends a startup message.

        let mut params = vec![ // Sets the display format for date and time values, as well as the rules for interpreting ambiguous date input values.  ("DateStyle", "ISO, MDY"),
            // Sets the client-side encoding (character set).
            // <https://www.postgresql.org/docs/devel/multibyte.html#MULTIBYTE-CHARSET-SUPPORTED>
            ("client_encoding", "UTF8"),
            // Sets the time zone for displaying and interpreting time stamps.
            ("TimeZone", "UTC"),
            // Adjust postgres to return precise values for floats
            // NOTE: This is default in postgres 12+
            ("extra_float_digits", "3"),
        ];

        // if let Some(ref application_name) = $options.get_application_name() {
        //     params.push(("application_name", application_name));
        // }

        self_.write_packet(&Startup {
            username: $options.get_username(),
            database: $options.get_database(),
            params: &params,
        })?;

        // The server then uses this information and the contents of
        // its configuration files (such as pg_hba.conf) to determine whether the connection is
        // provisionally acceptable, and what additional
        // authentication is required (if any).

        let mut process_id = 0;
        let mut secret_key = 0;
        let transaction_status;

        loop {
            let message: Message = connect!($(@$blocking)? @packet self_);
            match message.r#type {
                MessageType::Authentication => match message.decode()? {
                    Authentication::Ok => {
                        // the authentication exchange is successfully completed
                        // do nothing; no more information is required to continue
                    }

                    Authentication::CleartextPassword => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password in clear-text form.

                        self_
                            .write_packet(&Password::Cleartext(
                                $options.get_password().unwrap_or_default(),
                            ))?;
                    }

                    Authentication::Md5Password(body) => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password (with user name) encrypted via MD5, then encrypted again
                        // using the 4-byte random salt specified in the
                        // [AuthenticationMD5Password] message.

                        self_
                            .write_packet(&Password::Md5 {
                                username: $options.get_username().unwrap_or_default(),
                                password: $options.get_password().unwrap_or_default(),
                                salt: body.salt,
                            })?;
                    }

                    Authentication::Sasl(body) => {
                        sasl_authenticate!($(@$blocking)? self_, $options, body)
                    }

                    method => {
                        return Err(Error::configuration_msg(format!(
                            "unsupported authentication method: {:?}",
                            method
                        )));
                    }
                },

                MessageType::BackendKeyData => {
                    // provides secret-key data that the frontend must save if it wants to be
                    // able to issue cancel requests later

                    let data: BackendKeyData = message.decode()?;

                    process_id = data.process_id;
                    secret_key = data.secret_key;
                }

                MessageType::ReadyForQuery => {
                    let ready: ReadyForQuery = message.decode()?;

                    // start-up is completed. The frontend can now issue commands
                    transaction_status = ready.transaction_status;

                    break;
                }

                _ => {
                    return Err(Error::configuration_msg(format!(
                        "establish: unexpected message: {:?}",
                        message.r#type
                    )))
                }
            }
        }

        Ok(self_)
    }};
}

impl<Rt> PostgresConnection<Rt>
where
    Rt: sqlx_core::Runtime,
{
    #[cfg(feature = "async")]
    pub(crate) async fn connect_async(options: &PostgresConnectOptions<Rt>) -> Result<Self>
    where
        Rt: sqlx_core::Async,
    {
        connect!(options)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn connect(options: &PostgresConnectOptions<Rt>) -> Result<Self>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        connect!(@blocking options)
    }
}
