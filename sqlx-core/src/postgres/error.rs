use crate::postgres::protocol::Response;
use std::fmt::{self, Debug, Display};

pub struct PgError(pub(super) Box<Response>);

impl crate::error::DatabaseError for PgError {
    fn message(&self) -> &str {
        &self.0.message
    }

    fn details(&self) -> Option<&str> {
        self.0.detail.as_ref().map(|s| &**s)
    }

    fn hint(&self) -> Option<&str> {
        self.0.hint.as_ref().map(|s| &**s)
    }

    fn table_name(&self) -> Option<&str> {
        self.0.table.as_ref().map(|s| &**s)
    }

    fn column_name(&self) -> Option<&str> {
        self.0.column.as_ref().map(|s| &**s)
    }

    fn constraint_name(&self) -> Option<&str> {
        self.0.constraint.as_ref().map(|s| &**s)
    }
}

impl Debug for PgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::error::DatabaseError;

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

impl Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::error::DatabaseError;

        f.pad(self.message())
    }
}
