use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use sqlx_core::DatabaseError;

use crate::protocol::ErrPacket;

/// An error returned from the MySQL database server.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct MySqlDatabaseError(pub(crate) ErrPacket);

impl DatabaseError for MySqlDatabaseError {
    fn message(&self) -> &str {
        &self.0.error_message
    }
}

impl Display for MySqlDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0.sql_state {
            Some(state) => write!(f, "{} ({}): {}", self.0.error_code, state, self.message()),
            None => write!(f, "{}: {}", self.0.error_code, self.message()),
        }
    }
}

impl StdError for MySqlDatabaseError {}
