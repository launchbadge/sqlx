use std::fmt::{self, Debug, Display};

use crate::error::DatabaseError;
use crate::mysql::protocol::ErrPacket;

pub struct MySqlError(pub(super) ErrPacket);

impl DatabaseError for MySqlError {
    fn message(&self) -> &str {
        &*self.0.error_message
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        None
    }

    fn column_name(&self) -> Option<&str> {
        None
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }
}

// TODO: De-duplicate these two impls with Postgres (macro?)

impl Debug for MySqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DatabaseError")
            .field("message", &self.message())
            .field("details", &self.details())
            .field("hint", &self.hint())
            .field("table_name", &self.table_name())
            .field("column_name", &self.column_name())
            .field("constraint_name", &self.constraint_name())
            .finish()
    }
}

impl Display for MySqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}
