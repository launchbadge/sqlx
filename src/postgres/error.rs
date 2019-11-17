use super::protocol::Response;
use crate::error::DatabaseError;
use std::fmt::{self, Debug, Display};

#[derive(Debug)]
pub struct PostgresDatabaseError(pub(super) Box<Response>);

#[derive(Debug)]
pub struct ProtocolError<T>(pub(super) T);

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

impl<T: AsRef<str> + Debug + Send + Sync> DatabaseError for ProtocolError<T> {
    fn message(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T: AsRef<str>> Display for ProtocolError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.0.as_ref())
    }
}
