use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::path::PathBuf;

use either::Either;
use sqlx_core::{ConnectOptions, Runtime};

use crate::MySqlConnection;

mod builder;
mod default;
mod getters;
mod parse;

// TODO: RSA Public Key (to avoid the key exchange for caching_sha2 and sha256 plugins)

/// Options which can be used to configure how a MySQL connection is opened.
///
#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    runtime: PhantomData<Rt>,
    pub(crate) address: Either<(String, u16), PathBuf>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    timezone: String,
    charset: String,
}

impl<Rt> Clone for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn clone(&self) -> Self {
        Self {
            runtime: PhantomData,
            address: self.address.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            database: self.database.clone(),
            timezone: self.timezone.clone(),
            charset: self.charset.clone(),
        }
    }
}

impl<Rt> Debug for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
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

impl<Rt> ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    type Connection = MySqlConnection<Rt>;

    #[cfg(feature = "async")]
    fn connect(&self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<Self::Connection>>
    where
        Self::Connection: Sized,
        Rt: sqlx_core::Async,
    {
        Box::pin(MySqlConnection::<Rt>::connect_async(self))
    }
}

#[cfg(feature = "blocking")]
mod blocking {
    use sqlx_core::blocking::{ConnectOptions, Runtime};

    use super::{MySqlConnectOptions, MySqlConnection};

    impl<Rt: Runtime> ConnectOptions<Rt> for MySqlConnectOptions<Rt> {
        fn connect(&self) -> sqlx_core::Result<Self::Connection>
        where
            Self::Connection: Sized,
        {
            <MySqlConnection<Rt>>::connect(self)
        }
    }
}
