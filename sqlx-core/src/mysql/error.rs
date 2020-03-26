use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::error::DatabaseError;
use crate::mysql::protocol::ErrPacket;
use crate::mysql::MySql;

#[derive(Debug)]
pub struct MySqlError(pub(super) ErrPacket);

impl Display for MySqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}

impl DatabaseError for MySqlError {
    fn message(&self) -> &str {
        &*self.0.error_message
    }

    fn code(&self) -> Option<&str> {
        self.0.sql_state.as_deref()
    }

    fn as_ref_err(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_mut_err(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_box_err(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }
}

impl StdError for MySqlError {}

impl From<MySqlError> for crate::Error {
    fn from(err: MySqlError) -> Self {
        crate::Error::Database(Box::new(err))
    }
}
