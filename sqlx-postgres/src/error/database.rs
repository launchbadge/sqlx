use sqlx_core::{DatabaseError, Error};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use crate::PgNotice;

/// An error returned from the PostgreSQL database.
///
/// In PostgreSQL, an error is a [`PgNotice`] with a severity
/// at [`Error`][crate::PgNoticeSeverity::Error] or higher.
///
#[derive(Debug)]
pub struct PgDatabaseError(pub(crate) PgNotice);

impl Display for PgDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for PgDatabaseError {}

impl DatabaseError for PgDatabaseError {
    fn message(&self) -> &str {
        self.0.message()
    }
}

impl From<PgDatabaseError> for Error {
    fn from(err: PgDatabaseError) -> Self {
        Self::database(err)
    }
}
