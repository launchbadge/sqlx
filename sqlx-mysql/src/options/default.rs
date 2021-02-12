use either::Either;

use crate::MySqlConnectOptions;

pub(crate) const HOST: &str = "localhost";
pub(crate) const PORT: u16 = 3306;

impl Default for MySqlConnectOptions {
    fn default() -> Self {
        Self {
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

impl super::MySqlConnectOptions {
    /// Creates a default set of options ready for configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
