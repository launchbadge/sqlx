use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::error::DatabaseError;
use crate::mysql::protocol::ErrPacket;
use crate::mysql::MySql;

#[derive(Debug)]
pub struct MySqlDatabaseError(pub(super) ErrPacket);

impl Display for MySqlDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}

impl DatabaseError for MySqlDatabaseError {
    fn message(&self) -> &str {
        &*self.0.error_message
    }

    fn code(&self) -> Option<&str> {
        self.0.sql_state.as_deref()
    }
}

impl StdError for MySqlDatabaseError {}

impl From<MySqlDatabaseError> for crate::Error<MySql> {
    fn from(err: MySqlDatabaseError) -> Self {
        crate::Error::Database(Box::new(err))
    }
}
