use super::Connection;
use futures::StreamExt;
use sqlx_core::ConnectOptions;
use sqlx_postgres_protocol::{Authentication, Message, PasswordMessage, StartupMessage};
use std::io;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> io::Result<()> {
    // See this doc for more runtime parameters
    // https://www.postgresql.org/docs/12/runtime-config-client.html
    let mut message = StartupMessage::builder();

    if let Some(user) = options.user {
        // FIXME: User is technically required. We should default this like psql does.
        message = message.param("user", user);
    }

    if let Some(database) = options.database {
        message = message.param("database", database);
    }

    let message = message
        // Sets the display format for date and time values,
        // as well as the rules for interpreting ambiguous date input values.
        .param("DateStyle", "ISO, MDY")
        // Sets the display format for interval values.
        .param("IntervalStyle", "iso_8601")
        // Sets the time zone for displaying and interpreting time stamps.
        .param("TimeZone", "UTC")
        // Adjust postgres to return percise values for floats
        // NOTE: This is default in postgres 12+
        .param("extra_float_digits", "3")
        // Sets the client-side encoding (character set).
        .param("client_encoding", "UTF-8")
        .build();

    conn.send(message).await?;

    // FIXME: This feels like it could be reduced (see other connection flows)
    while let Some(message) = conn.incoming.next().await {
        match message {
            Message::Authentication(Authentication::Ok) => {
                // Do nothing; server is just telling us that
                // there is no password needed
            }

            Message::Authentication(Authentication::CleartextPassword) => {
                // FIXME: Should error early (before send) if the user did not supply a password
                conn.send(PasswordMessage::cleartext(
                    options.password.unwrap_or_default(),
                ))
                .await?;
            }

            Message::Authentication(Authentication::Md5Password { salt }) => {
                // FIXME: Should error early (before send) if the user did not supply a password
                conn.send(PasswordMessage::md5(
                    options.password.unwrap_or_default(),
                    options.user.unwrap_or_default(),
                    &salt,
                ))
                .await?;
            }

            Message::BackendKeyData(body) => {
                conn.process_id = body.process_id();
                conn.secret_key = body.secret_key();
            }

            Message::ReadyForQuery(_) => {
                break;
            }

            _ => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    Ok(())
}
