use super::protocol::Response;
use crate::error::DatabaseError;
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug)]
pub struct PostgresDatabaseError(pub(super) Box<Response>);

#[derive(Debug)]
pub struct ProtocolError<T>(pub(super) T);

impl DatabaseError for PostgresDatabaseError {
    fn message(&self) -> &str {
        self.0.message()
    }
}

impl<T: AsRef<str> + Debug + Send + Sync> DatabaseError for ProtocolError<T> {
    fn message(&self) -> &str {
        self.0.as_ref()
    }
}
