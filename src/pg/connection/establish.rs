use super::PgRawConnection;
use crate::pg::protocol::{Authentication, Message, PasswordMessage, StartupMessage};
use std::io;
use url::Url;

pub async fn establish<'a, 'b: 'a>(conn: &'a mut PgRawConnection, url: &'b Url) -> io::Result<()> {
    let user = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

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

    let message = StartupMessage { params };

    conn.write(message);
    conn.flush().await?;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::Authentication(auth) => {
                match *auth {
                    Authentication::Ok => {
                        // Do nothing. No password is needed to continue.
                    }

                    Authentication::CleartextPassword => {
                        // FIXME: Should error early (before send) if the user did not supply a password
                        conn.write(PasswordMessage::Cleartext(password));

                        conn.flush().await?;
                    }

                    Authentication::Md5Password { salt } => {
                        // FIXME: Should error early (before send) if the user did not supply a password
                        conn.write(PasswordMessage::Md5 {
                            password,
                            user,
                            salt,
                        });

                        conn.flush().await?;
                    }

                    auth => {
                        unimplemented!("received {:?} unimplemented authentication message", auth);
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
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    Ok(())
}
