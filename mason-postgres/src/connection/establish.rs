use super::Connection;
use crate::protocol::{
    client::{PasswordMessage, StartupMessage},
    server::Message as ServerMessage,
};
use futures::StreamExt;
use mason_core::ConnectOptions;
use md5::{Digest, Md5};
use std::io;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> io::Result<()> {
    // See this doc for more runtime parameters
    // https://www.postgresql.org/docs/12/runtime-config-client.html
    let params = [
        ("user", options.user),
        ("database", options.database),
        // TODO: Expose this property perhaps?
        (
            "application_name",
            Some(concat!(env!("CARGO_PKG_NAME"), "/v", env!("CARGO_PKG_VERSION"))),
        ),
        // Sets the display format for date and time values,
        // as well as the rules for interpreting ambiguous date input values.
        ("DateStyle", Some("ISO, MDY")),
        // Sets the display format for interval values.
        ("IntervalStyle", Some("iso_8601")),
        // Sets the time zone for displaying and interpreting time stamps.
        ("TimeZone", Some("UTC")),
        // Adjust postgres to return percise values for floats
        // NOTE: This is default in postgres 12+
        ("extra_float_digits", Some("3")),
        // Sets the client-side encoding (character set).
        ("client_encoding", Some("UTF-8")),
    ];

    conn.send(StartupMessage { params: &params }).await?;

    while let Some(message) = conn.incoming.next().await {
        match message {
            ServerMessage::AuthenticationOk => {
                // Do nothing; server is just telling us that
                // there is no password needed
            }

            ServerMessage::AuthenticationCleartextPassword => {
                conn.send(PasswordMessage { password: options.password.unwrap_or_default() })
                    .await?;
            }

            ServerMessage::AuthenticationMd5Password(body) => {
                // Hash password|username
                // FIXME: ConnectOptions should prepare a default user
                let pass_user =
                    md5(options.password.unwrap_or_default(), options.user.unwrap_or_default());

                let with_salt = md5(pass_user, body.salt());
                let password = format!("md5{}", with_salt);

                conn.send(PasswordMessage { password: &password }).await?;
            }

            ServerMessage::BackendKeyData(body) => {
                conn.process_id = body.process_id();
                conn.secret_key = body.secret_key();
            }

            ServerMessage::ReadyForQuery(_) => {
                // Good to go
                break;
            }

            _ => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    Ok(())
}

#[inline]
fn md5(a: impl AsRef<[u8]>, b: impl AsRef<[u8]>) -> String {
    hex::encode(Md5::new().chain(a).chain(b).result())
}
