use std::fmt::{self, Display};

use crate::error::DatabaseError;
use crate::mysql::protocol::ErrPacket;

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
}
