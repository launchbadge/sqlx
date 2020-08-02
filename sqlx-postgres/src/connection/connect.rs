use crate::codec::backend::{Authentication, BackendKeyData, MessageFormat, ReadyForQuery};
use crate::codec::frontend;
use crate::{PgConnectOptions, PgConnection};
use sqlx_core::{error::Error, io::BufStream};
use sqlx_rt::TcpStream;

impl PgConnection {
    pub(crate) async fn connect(options: &PgConnectOptions) -> Result<Self, Error> {
        let stream = TcpStream::connect((&*options.host, options.port)).await?;

        // Set TCP_NODELAY to disable the Nagle algorithm
        // We are telling the kernel that we bundle data to be sent in large write() calls
        // instead of sending many small packets.
        stream.set_nodelay(true)?;

        // TODO: Upgrade to TLS if asked

        let mut stream = BufStream::with_capacity(stream, 1024, 1024);

        // To begin a session, a frontend opens a connection to the server
        // and sends a startup message.

        stream.write(frontend::Startup(&[
            ("user", Some(&options.username)),
            ("database", options.database.as_deref()),
            // Sets the display format for date and time values,
            // as well as the rules for interpreting ambiguous date input values.
            ("DateStyle", Some("ISO, MDY")),
            //
            // Sets the client-side encoding (character set).
            // <https://www.postgresql.org/docs/devel/multibyte.html#MULTIBYTE-CHARSET-SUPPORTED>
            ("client_encoding", Some("UTF8")),
            //
            // Sets the time zone for displaying and interpreting time stamps.
            ("TimeZone", Some("UTC")),
            //
            // Adjust postgres to return (more) precise values for floats
            // NOTE: This is default in postgres 12+
            ("extra_float_digits", Some("3")),
        ]))?;

        // Wrap our network in the connection type with default values for its properties
        // This lets us access methods on self

        let mut conn = Self::new(stream);

        // The server then uses this information and the contents of
        // its configuration files (such as pg_hba.conf) to determine whether the connection is
        // provisionally acceptable, and what additional
        // authentication is required (if any).

        loop {
            let message = conn.recv().await?;

            match message.format {
                MessageFormat::Authentication => match message.decode()? {
                    Authentication::Ok => {
                        // the authentication exchange is successfully completed
                        // do nothing; no more information is required to continue
                    }

                    Authentication::CleartextPassword => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password in clear-text form.

                        conn.stream.write(frontend::Password::Cleartext(
                            options.password.as_deref().unwrap_or_default(),
                        ))?;
                    }

                    Authentication::Md5Password(body) => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password (with user name) encrypted via MD5, then encrypted again
                        // using the 4-byte random salt specified in the
                        // [AuthenticationMD5Password] message.

                        conn.stream.write(frontend::Password::Md5 {
                            username: &options.username,
                            password: options.password.as_deref().unwrap_or_default(),
                            salt: body.salt,
                        })?;
                    }

                    // Authentication::Sasl(body) => {
                    //     // sasl::authenticate(&mut stream, options, body).await?;
                    //     todo!("sasl")
                    // }
                    method => {
                        return Err(Error::protocol_msg(format!(
                            "unsupported authentication method: {:?}",
                            method
                        )));
                    }
                },

                MessageFormat::BackendKeyData => {
                    // provides secret-key data that the frontend must save if it wants to be
                    // able to issue cancel requests later

                    let data: BackendKeyData = message.decode()?;

                    conn.process_id = data.process_id;
                    conn.secret_key = data.secret_key;
                }

                MessageFormat::ReadyForQuery => {
                    conn.transaction_status = message.decode::<ReadyForQuery>()?.transaction_status;

                    // start-up is completed.
                    // the frontend can now issue commands.
                    break;
                }

                _ => {}
            }
        }

        Ok(conn)
    }
}
