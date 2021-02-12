use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::path::PathBuf;

use either::Either;
use sqlx_core::{ConnectOptions, Result, Runtime};

use crate::MySqlConnection;

mod builder;
mod default;
mod getters;
mod parse;

// TODO: RSA Public Key (to avoid the key exchange for caching_sha2 and sha256 plugins)

/// Options which can be used to configure how a MySQL connection is opened.
///
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnectOptions {
    pub(crate) address: Either<(String, u16), PathBuf>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    timezone: String,
    charset: String,
}

impl Clone for MySqlConnectOptions {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            database: self.database.clone(),
            timezone: self.timezone.clone(),
            charset: self.charset.clone(),
        }
    }
}

impl Debug for MySqlConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnectOptions")
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
            .field("timezone", &self.timezone)
            .field("charset", &self.charset)
            .finish()
    }
}

impl ConnectOptions for MySqlConnectOptions {}

#[cfg(feature = "blocking")]
mod blocking {
    use sqlx_core::blocking::ConnectOptions;

    use super::MySqlConnectOptions;

    impl ConnectOptions for MySqlConnectOptions {}
}
