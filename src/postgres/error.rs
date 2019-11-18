use super::protocol::Response;
use crate::error::DatabaseError;
use std::fmt::{self, Debug, Display};

#[derive(Debug)]
pub struct PostgresDatabaseError(pub(super) Box<Response>);

impl DatabaseError for PostgresDatabaseError {
    fn message(&self) -> &str {
        self.0.message()
    }
}

impl Display for PostgresDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}
