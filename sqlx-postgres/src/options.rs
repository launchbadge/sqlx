use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;

use either::Either;
use sqlx_core::ConnectOptions;

mod builder;
mod default;
mod getters;
mod parse;

/// Options which can be used to configure how a Postgres connection is opened.
///
/// A value of `PgConnectOptions` can be parsed from a connection URI, as
/// described by [libpq](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING).
///
/// ```text
/// postgresql://[user[:password]@][host][:port][/database][?param1=value1&...]
/// ```
///
#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct PgConnectOptions {
    pub(crate) address: Either<(String, u16), PathBuf>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    application_name: Option<String>,
}

impl Debug for PgConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnectOptions")
            .field(
                "address",
                &self
                    .address
                    .as_ref()
                    .map_left(|(host, port)| format!("{}:{}", host, port))
                    .map_right(|socket| socket.display()),
            )
            .field("username", &self.username)
            .field("password", &self.password)
            .field("database", &self.database)
            .field("application_name", &self.application_name)
            .finish()
    }
}

impl ConnectOptions for PgConnectOptions {}

#[cfg(feature = "blocking")]
impl sqlx_core::blocking::ConnectOptions for PgConnectOptions {}
