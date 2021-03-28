//! Implements start-up flow.
//!
//! To begin a session, a frontend opens a connection to the server
//! and sends a startup message.
//!
//! The server then sends an appropriate authentication request message, to
//! which the frontend must reply with an appropriate authentication
//! response message.
//!
//! The authentication cycle ends with the server either rejecting
//! the connection attempt (ErrorResponse), or sending AuthenticationOk.
//!
//! <https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.3>

use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Result, Runtime};

use crate::protocol::backend::{Authentication, BackendMessage, BackendMessageType, KeyData};
use crate::protocol::frontend::{Password, PasswordMd5, Startup};
use crate::{PgClientError, PgConnectOptions, PgConnection};

impl<Rt: Runtime> PgConnection<Rt> {
    fn write_startup_message(&mut self, options: &PgConnectOptions) -> Result<()> {
        let params = vec![
            ("user", options.get_username()),
            ("database", options.get_database()),
            ("application_name", options.get_application_name()),
            // sets the text display format for date and time values
            // as well as the rules for interpreting ambiguous date input values
            ("DateStyle", Some("ISO, MDY")),
            // sets the client-side encoding (charset)
            // NOTE: this must not be changed, too much in the driver depends on this being set to UTF-8
            ("client_encoding", Some("UTF8")),
            // sets the timezone for displaying and interpreting time stamps
            // NOTE: this is only used to assume timestamptz values are in UTC
            ("TimeZone", Some("UTC")),
        ];

        self.stream.write_message(&Startup(&params))
    }

    fn handle_startup_response(
        &mut self,
        options: &PgConnectOptions,
        message: BackendMessage,
    ) -> Result<bool> {
        match message.ty {
            BackendMessageType::Authentication => match message.deserialize()? {
                Authentication::Ok => {
                    // nothing more to do with authentication
                }

                Authentication::Md5Password(data) => {
                    self.stream.write_message(&PasswordMd5 {
                        password: options.get_password().unwrap_or_default(),
                        username: options.get_username().unwrap_or_default(),
                        salt: data.salt,
                    })?;
                }

                Authentication::CleartextPassword => {
                    self.stream
                        .write_message(&Password(options.get_password().unwrap_or_default()))?;
                }

                Authentication::Sasl(_) => todo!("sasl"),
                Authentication::SaslContinue(_) => todo!("sasl continue"),
                Authentication::SaslFinal(_) => todo!("sasl final"),
            },

            BackendMessageType::ReadyForQuery => {
                self.handle_ready_for_query(message.deserialize()?);

                // fully connected
                return Ok(true);
            }

            BackendMessageType::BackendKeyData => {
                let key_data: KeyData = message.deserialize()?;

                self.process_id = key_data.process_id;
                self.secret_key = key_data.secret_key;
            }

            ty => {
                return Err(PgClientError::UnexpectedMessageType {
                    ty: ty as u8,
                    context: "starting up",
                }
                .into());
            }
        }

        Ok(false)
    }
}

macro_rules! impl_connect {
    (@blocking @new $options:ident) => {
        NetStream::connect($options.address.as_ref())?
    };

    (@new $options:ident) => {
        NetStream::connect_async($options.address.as_ref()).await?
    };

    ($(@$blocking:ident)? $options:ident) => {{
        // open a network stream to the database server
        let stream = impl_connect!($(@$blocking)? @new $options);

        // construct a <PgConnection> around the network stream
        // wraps the stream in a <BufStream> to buffer read and write
        let mut self_ = Self::new(stream);

        // to begin a session, a frontend should send a startup message
        // this is built up of various startup parameters that control the connection
        self_.write_startup_message($options)?;
        self_.pending_ready_for_query_count += 1;

        // the server then uses this information and the contents of
        // its configuration files (such as pg_hba.conf) to determine whether the connection is
        // provisionally acceptable, and what additional
        // authentication is required (if any).
        loop {
            let message = read_message!($(@$blocking)? self_.stream)?;

            if self_.handle_startup_response($options, message)? {
                // complete, successful authentication
                break;
            }
        }

        Ok(self_)
    }};
}

impl<Rt: Runtime> PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn connect_async(options: &PgConnectOptions) -> Result<Self>
    where
        Rt: sqlx_core::Async,
    {
        impl_connect!(options)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn connect_blocking(options: &PgConnectOptions) -> Result<Self>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_connect!(@blocking options)
    }
}
