use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use sqlx_core::DatabaseError;

/// An error returned from the PostgreSQL database server.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct PostgresDatabaseError(String);

impl PostgresDatabaseError {
    pub(crate) fn protocol(msg: String) -> PostgresDatabaseError {
        PostgresDatabaseError(msg)
    }
}

impl From<crypto_mac::InvalidKeyLength> for PostgresDatabaseError {
    fn from(err: crypto_mac::InvalidKeyLength) -> Self {
        PostgresDatabaseError::protocol(err.to_string())
    }
}

impl From<crypto_mac::MacError> for PostgresDatabaseError {
    fn from(err: crypto_mac::MacError) -> Self {
        PostgresDatabaseError::protocol(err.to_string())
    }
}

impl DatabaseError for PostgresDatabaseError {
    fn message(&self) -> &str {
        &self.0
    }
}

impl Display for PostgresDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for PostgresDatabaseError {}
