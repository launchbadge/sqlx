use crate::connection::{sasl, stream::PgStream};
use crate::error::Error;
use crate::message::{Authentication, BackendKeyData, BackendMessageFormat, Password, Startup};
use crate::{PgConnectOptions, PgConnection};
use futures_channel::mpsc::unbounded;

use super::stream::parse_server_version;
use super::worker::Worker;

// https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.3
// https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.11

impl PgConnection {
    pub(crate) async fn establish(options: &PgConnectOptions) -> Result<Self, Error> {
        // Upgrade to TLS if we were asked to and the server supports it
        let stream = PgStream::connect(options).await?;

        let (notif_tx, notif_rx) = unbounded();

        let (channel, shared) = Worker::spawn(stream.into_inner(), notif_tx);

        let mut conn = PgConnection::new(options, channel, notif_rx, shared);

        // To begin a session, a frontend opens a connection to the server
        // and sends a startup message.

        let mut params = vec![
            // Sets the display format for date and time values,
            // as well as the rules for interpreting ambiguous date input values.
            ("DateStyle", "ISO, MDY"),
            // Sets the client-side encoding (character set).
            // <https://www.postgresql.org/docs/devel/multibyte.html#MULTIBYTE-CHARSET-SUPPORTED>
            ("client_encoding", "UTF8"),
            // Sets the time zone for displaying and interpreting time stamps.
            ("TimeZone", "UTC"),
        ];

        if let Some(ref extra_float_digits) = options.extra_float_digits {
            params.push(("extra_float_digits", extra_float_digits));
        }

        if let Some(ref application_name) = options.application_name {
            params.push(("application_name", application_name));
        }

        if let Some(ref options) = options.options {
            params.push(("options", options));
        }

        let mut pipe = conn.pipe(|buf| {
            buf.write(Startup {
                username: Some(&options.username),
                database: options.database.as_deref(),
                params: &params,
            })
        })?;

        // The server then uses this information and the contents of
        // its configuration files (such as pg_hba.conf) to determine whether the connection is
        // provisionally acceptable, and what additional
        // authentication is required (if any).

        let mut process_id = 0;
        let mut secret_key = 0;

        loop {
            let message = pipe.recv().await?;
            match message.format {
                BackendMessageFormat::Authentication => match message.decode()? {
                    Authentication::Ok => {
                        // the authentication exchange is successfully completed
                        // do nothing; no more information is required to continue
                    }

                    Authentication::CleartextPassword => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password in clear-text form.

                        conn.pipe_and_forget(Password::Cleartext(
                            options.password.as_deref().unwrap_or_default(),
                        ))?;
                    }

                    Authentication::Md5Password(body) => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password (with user name) encrypted via MD5, then encrypted again
                        // using the 4-byte random salt specified in the
                        // [AuthenticationMD5Password] message.

                        conn.pipe_and_forget(Password::Md5 {
                            username: &options.username,
                            password: options.password.as_deref().unwrap_or_default(),
                            salt: body.salt,
                        })?;
                    }

                    Authentication::Sasl(body) => {
                        sasl::authenticate(&conn, &mut pipe, options, body).await?;
                    }

                    method => {
                        return Err(err_protocol!(
                            "unsupported authentication method: {:?}",
                            method
                        ));
                    }
                },

                BackendMessageFormat::BackendKeyData => {
                    // provides secret-key data that the frontend must save if it wants to be
                    // able to issue cancel requests later

                    let data: BackendKeyData = message.decode()?;

                    process_id = data.process_id;
                    secret_key = data.secret_key;
                }

                BackendMessageFormat::ReadyForQuery => {
                    // Transaction status is updated in the bg worker.
                    break;
                }

                _ => {
                    return Err(err_protocol!(
                        "establish: unexpected message: {:?}",
                        message.format
                    ))
                }
            }
        }

        let server_version = conn
            .inner
            .shared
            .remove_parameter_status("server_version")
            .map(parse_server_version);

        conn.inner.server_version_num = server_version.flatten();
        conn.inner.secret_key = secret_key;
        conn.inner.process_id = process_id;

        Ok(conn)
    }
}
