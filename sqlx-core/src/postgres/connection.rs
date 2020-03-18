use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::Range;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::TryFutureExt;

use crate::connection::{Connect, Connection};
use crate::executor::Executor;
use crate::postgres::protocol::{
    Authentication, AuthenticationMd5, AuthenticationSasl, BackendKeyData, Message,
    PasswordMessage, StartupMessage, StatementId, Terminate, TypeFormat,
};
use crate::postgres::stream::PgStream;
use crate::postgres::{sasl, tls};
use crate::url::Url;

/// An asynchronous connection to a [Postgres][super::Postgres] database.
///
/// The connection string expected by [Connect::connect] should be a PostgreSQL connection
/// string, as documented at
/// <https://www.postgresql.org/docs/12/libpq-connect.html#LIBPQ-CONNSTRING>
///
/// ### TLS Support (requires `tls` feature)
/// This connection type supports the same `sslmode` query parameter that `libpq` does in
/// connection strings: <https://www.postgresql.org/docs/12/libpq-ssl.html>
///
/// ```text
/// postgresql://<user>[:<password>]@<host>[:<port>]/<database>[?sslmode=<ssl-mode>[&sslcrootcert=<path>]]
/// ```
/// where
/// ```text
/// ssl-mode = disable | allow | prefer | require | verify-ca | verify-full
/// path = percent (URL) encoded path on the local machine
/// ```
///
/// If the `tls` feature is not enabled, `disable`, `allow` and `prefer` are no-ops and `require`,
/// `verify-ca` and `verify-full` are forbidden (attempting to connect with these will return
/// an error).
///
/// If the `tls` feature is enabled, an upgrade to TLS is attempted on every connection by default
/// (equivalent to `sslmode=prefer`). If the server does not support TLS (because it was not
/// started with a valid certificate and key, see <https://www.postgresql.org/docs/12/ssl-tcp.html>)
/// then it falls back to an unsecured connection and logs a warning.
///
/// Add `sslmode=require` to your connection string to emit an error if the TLS upgrade fails.
///
/// If you're running Postgres locally, your connection string might look like this:
/// ```text
/// postgresql://root:password@localhost/my_database?sslmode=require
/// ```
///
/// However, like with `libpq` the server certificate is **not** checked for validity by default.
///
/// Specifying `sslmode=verify-ca` will cause the TLS upgrade to verify the server's SSL
/// certificate against a local CA root certificate; this is not the system root certificate
/// but is instead expected to be specified in one of a few ways:
///
/// * The path to the certificate can be specified by adding the `sslrootcert` query parameter
/// to the connection string. (Remember to percent-encode it!)
///
/// * The path may also be specified via the `PGSSLROOTCERT` environment variable (which
/// should *not* be percent-encoded.)
///
/// * Otherwise, the library will look for the Postgres global root CA certificate in the default
/// location:
///
///     * `$HOME/.postgresql/root.crt` on POSIX systems
///     * `%APPDATA%\postgresql\root.crt` on Windows
///
/// These locations are documented here: <https://www.postgresql.org/docs/12/libpq-ssl.html#LIBQ-SSL-CERTIFICATES>
/// If the root certificate cannot be found by any of these means then the TLS upgrade will fail.
///
/// If `sslmode=verify-full` is specified, in addition to checking the certificate as with
/// `sslmode=verify-ca`, the hostname in the connection string will be verified
/// against the hostname in the server certificate, so they must be the same for the TLS
/// upgrade to succeed.
pub struct PgConnection {
    pub(super) stream: PgStream,
    pub(super) next_statement_id: u32,
    pub(super) is_ready: bool,

    pub(super) cache_statement: HashMap<Box<str>, StatementId>,
    pub(super) cache_statement_columns: HashMap<StatementId, Arc<HashMap<Box<str>, usize>>>,
    pub(super) cache_statement_formats: HashMap<StatementId, Arc<[TypeFormat]>>,

    // Work buffer for the value ranges of the current row
    // This is used as the backing memory for each Row's value indexes
    pub(super) current_row_values: Vec<Option<Range<u32>>>,

    // TODO: Find a use for these values. Perhaps in a debug impl of PgConnection?
    #[allow(dead_code)]
    process_id: u32,

    #[allow(dead_code)]
    secret_key: u32,
}

// https://www.postgresql.org/docs/12/protocol-flow.html#id-1.10.5.7.3
async fn startup(stream: &mut PgStream, url: &Url) -> crate::Result<BackendKeyData> {
    // Defaults to postgres@.../postgres
    let username = url.username().unwrap_or("postgres");
    let database = url.database().unwrap_or("postgres");

    // See this doc for more runtime parameters
    // https://www.postgresql.org/docs/12/runtime-config-client.html
    let params = &[
        ("user", username),
        ("database", database),
        // Sets the display format for date and time values,
        // as well as the rules for interpreting ambiguous date input values.
        ("DateStyle", "ISO, MDY"),
        // Sets the time zone for displaying and interpreting time stamps.
        ("TimeZone", "UTC"),
        // Adjust postgres to return percise values for floats
        // NOTE: This is default in postgres 12+
        ("extra_float_digits", "3"),
        // Sets the client-side encoding (character set).
        ("client_encoding", "UTF-8"),
    ];

    stream.write(StartupMessage { params });
    stream.flush().await?;

    let mut key_data = BackendKeyData {
        process_id: 0,
        secret_key: 0,
    };

    loop {
        match stream.receive().await? {
            Message::Authentication => match Authentication::read(stream.buffer())? {
                Authentication::Ok => {
                    // do nothing. no password is needed to continue.
                }

                Authentication::CleartextPassword => {
                    stream.write(PasswordMessage::ClearText(
                        &url.password().unwrap_or_default(),
                    ));

                    stream.flush().await?;
                }

                Authentication::Md5Password => {
                    // TODO: Just reference the salt instead of returning a stack array
                    // TODO: Better way to make sure we skip the first 4 bytes here
                    let data = AuthenticationMd5::read(&stream.buffer()[4..])?;

                    stream.write(PasswordMessage::Md5 {
                        password: &url.password().unwrap_or_default(),
                        user: username,
                        salt: data.salt,
                    });

                    stream.flush().await?;
                }

                Authentication::Sasl => {
                    // TODO: Make this iterative for traversing the mechanisms to remove the allocation
                    // TODO: Better way to make sure we skip the first 4 bytes here
                    let data = AuthenticationSasl::read(&stream.buffer()[4..])?;

                    let mut has_sasl: bool = false;
                    let mut has_sasl_plus: bool = false;

                    for mechanism in &*data.mechanisms {
                        match &**mechanism {
                            "SCRAM-SHA-256" => {
                                has_sasl = true;
                            }

                            "SCRAM-SHA-256-PLUS" => {
                                has_sasl_plus = true;
                            }

                            _ => {
                                log::info!("unsupported auth mechanism: {}", mechanism);
                            }
                        }
                    }

                    if has_sasl || has_sasl_plus {
                        // TODO: Handle -PLUS differently if we're in a TLS stream
                        sasl::authenticate(stream, username, &url.password().unwrap_or_default())
                            .await?;
                    } else {
                        return Err(protocol_err!(
                            "unsupported SASL auth mechanisms: {:?}",
                            data.mechanisms
                        )
                        .into());
                    }
                }

                auth => {
                    return Err(
                        protocol_err!("requested unsupported authentication: {:?}", auth).into(),
                    );
                }
            },

            Message::BackendKeyData => {
                // do nothing. we do not care about the server values here.
                key_data = BackendKeyData::read(stream.buffer())?;
            }

            Message::ParameterStatus => {
                // do nothing. we do not care about the server values here.
            }

            Message::ReadyForQuery => {
                // done. connection is now fully established and can accept
                // queries for execution.
                break;
            }

            type_ => {
                return Err(protocol_err!("unexpected message: {:?}", type_).into());
            }
        }
    }

    Ok(key_data)
}

// https://www.postgresql.org/docs/12/protocol-flow.html#id-1.10.5.7.10
async fn terminate(mut stream: PgStream) -> crate::Result<()> {
    stream.write(Terminate);
    stream.flush().await?;
    stream.shutdown()?;

    Ok(())
}

impl PgConnection {
    pub(super) async fn new(url: crate::Result<Url>) -> crate::Result<Self> {
        let url = url?;
        let mut stream = PgStream::new(&url).await?;

        tls::request_if_needed(&mut stream, &url).await?;
        let key_data = startup(&mut stream, &url).await?;

        Ok(Self {
            stream,
            current_row_values: Vec::with_capacity(10),
            next_statement_id: 1,
            is_ready: true,
            cache_statement: HashMap::new(),
            cache_statement_columns: HashMap::new(),
            cache_statement_formats: HashMap::new(),
            process_id: key_data.process_id,
            secret_key: key_data.secret_key,
        })
    }
}

impl Connect for PgConnection {
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<PgConnection>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(PgConnection::new(url.try_into()))
    }
}

impl Connection for PgConnection {
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(terminate(self.stream))
    }

    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        Box::pin(Executor::execute(self, "SELECT 1").map_ok(|_| ()))
    }
}
