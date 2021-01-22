use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use sqlx_core::DatabaseError;

/// An error returned from the PostgreSQL database server.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct PostgresDatabaseError();

impl DatabaseError for PostgresDatabaseError {
    fn message(&self) -> &str {
        todo!()
    }
}

impl Display for PostgresDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "TODO")
    }
}

impl StdError for PostgresDatabaseError {}
