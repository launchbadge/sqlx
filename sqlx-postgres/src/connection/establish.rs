use super::Connection;
use sqlx_core::ConnectOptions;
use sqlx_postgres_protocol::{Authentication, Message, PasswordMessage, StartupMessage};
use std::io;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> io::Result<()> {
    // See this doc for more runtime parameters
    // https://www.postgresql.org/docs/12/runtime-config-client.html
    let params = &[
        // FIXME: ConnectOptions user and database need to be required parameters and error
        //        before they get here
        ("user", options.user.expect("user is required")),
        ("database", options.database.expect("database is required")),
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

    let message = StartupMessage::new(params);

    conn.send(message);
    conn.flush().await?;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::Authentication(Authentication::Ok) => {
                // Do nothing; server is just telling us that
                // there is no password needed
            }

            Message::Authentication(Authentication::CleartextPassword) => {
                // FIXME: Should error early (before send) if the user did not supply a password
                conn.send(PasswordMessage::cleartext(
                    options.password.unwrap_or_default(),
                ));
                conn.flush().await?;
            }

            Message::Authentication(Authentication::Md5Password { salt }) => {
                // FIXME: Should error early (before send) if the user did not supply a password
                conn.send(PasswordMessage::md5(
                    options.password.unwrap_or_default(),
                    options.user.unwrap_or_default(),
                    salt,
                ));
                conn.flush().await?;
            }

            Message::BackendKeyData(body) => {
                conn.process_id = body.process_id();
                conn.secret_key = body.secret_key();
            }

            Message::ReadyForQuery(_) => {
                break;
            }

            message => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    Ok(())
}
