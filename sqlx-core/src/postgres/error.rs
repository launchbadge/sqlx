use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::error::DatabaseError;
use crate::postgres::protocol::Response;
use crate::postgres::Postgres;

#[derive(Debug)]
pub struct PgDatabaseError(pub(super) Response);

impl DatabaseError for PgDatabaseError {
    fn message(&self) -> &str {
        &self.0.message
    }

    fn code(&self) -> Option<&str> {
        Some(&self.0.code)
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

impl Display for PgDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}

impl StdError for PgDatabaseError {}

impl From<PgDatabaseError> for crate::Error<Postgres> {
    fn from(err: PgDatabaseError) -> Self {
        crate::Error::Database(Box::new(err))
    }
}
