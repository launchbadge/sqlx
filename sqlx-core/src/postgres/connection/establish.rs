use crate::error::Error;
use crate::io::{BufStream, Decode};
use crate::net::{MaybeTlsStream, Socket};
use crate::postgres::connection::stream::PgStream;
use crate::postgres::message::{
    Authentication, BackendKeyData, MessageFormat, Password, ReadyForQuery, Startup,
    TransactionStatus,
};
use crate::postgres::{PgConnectOptions, PgConnection};
use hashbrown::HashMap;

// https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.3
// https://www.postgresql.org/docs/current/protocol-flow.html#id-1.10.5.7.11

impl PgConnection {
    pub(crate) async fn establish(options: &PgConnectOptions) -> Result<Self, Error> {
        let mut stream = PgStream::connect(options).await?;

        // To begin a session, a frontend opens a connection to the server
        // and sends a startup message.

        stream
            .write(Startup {
                username: Some(&options.username),
                database: options.database.as_deref(),
                params: &[
                    // Sets the client-side encoding (character set).
                    // <https://www.postgresql.org/docs/devel/multibyte.html#MULTIBYTE-CHARSET-SUPPORTED>
                    ("client_encoding", "UTF8"),
                    // Sets the time zone for displaying and interpreting time stamps.
                    ("TimeZone", "UTC"),
                ],
            })
            .await?;

        stream.flush().await?;

        // The server then uses this information and the contents of
        // its configuration files (such as pg_hba.conf) to determine whether the connection is
        // provisionally acceptable, and what additional
        // authentication is required (if any).

        let mut process_id = 0;
        let mut secret_key = 0;
        let mut transaction_status = TransactionStatus::Idle;

        loop {
            let message = stream.recv().await?;
            match message.format {
                MessageFormat::Authentication => match message.decode()? {
                    Authentication::Ok => {
                        // the authentication exchange is successfully completed
                        // do nothing; no more information is required to continue
                    }

                    Authentication::CleartextPassword => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password in clear-text form.

                        stream
                            .write(Password::Cleartext(
                                options.password.as_deref().unwrap_or_default(),
                            ))
                            .await?;

                        stream.flush().await?;
                    }

                    Authentication::Md5Password(body) => {
                        // The frontend must now send a [PasswordMessage] containing the
                        // password (with user name) encrypted via MD5, then encrypted again
                        // using the 4-byte random salt specified in the
                        // [AuthenticationMD5Password] message.

                        stream
                            .write(Password::Md5 {
                                username: &options.username,
                                password: options.password.as_deref().unwrap_or_default(),
                                salt: body.salt,
                            })
                            .await?;

                        stream.flush().await?;
                    }

                    method => {
                        return Err(err_protocol!(
                            "unsupported authentication method: {:?}",
                            method
                        ));
                    }
                },

                MessageFormat::ParameterStatus => {
                    // informs the frontend about the current (initial)
                    // setting of backend parameters

                    // we currently have no use for that data so we promptly ignore this message
                }

                MessageFormat::BackendKeyData => {
                    // provides secret-key data that the frontend must save if it wants to be
                    // able to issue cancel requests later

                    let data: BackendKeyData = message.decode()?;

                    process_id = data.process_id;
                    secret_key = data.secret_key;
                }

                MessageFormat::ReadyForQuery => {
                    // start-up is completed. The frontend can now issue commands
                    transaction_status =
                        ReadyForQuery::decode(message.contents)?.transaction_status;

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

        Ok(PgConnection {
            stream,
            process_id,
            secret_key,
            transaction_status,
            pending_ready_for_query_count: 0,
            next_statement_id: (1..),
            cache_statement: HashMap::new(),
            cache_type_id: HashMap::new(),
            cache_type_name: HashMap::new(),
        })
    }
}
