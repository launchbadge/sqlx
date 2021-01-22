use std::marker::PhantomData;

use either::Either;
use sqlx_core::Runtime;

use crate::PostgresConnectOptions;

pub(crate) const HOST: &str = "localhost";
pub(crate) const PORT: u16 = 3306;

impl<Rt> Default for PostgresConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn default() -> Self {
        Self {
            runtime: PhantomData,
            address: Either::Left((HOST.to_owned(), PORT)),
            username: None,
            password: None,
            database: None,
            charset: "utf8mb4".to_owned(),
            timezone: "utc".to_owned(),
            // todo: connect_timeout
        }
    }
}

impl<Rt> super::PostgresConnectOptions<Rt>
where
    Rt: Runtime,
{
    /// Creates a default set of options ready for configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
