use super::protocol::Response;
use crate::error::DatabaseError;

#[derive(Debug)]
pub struct PostgresDatabaseError(pub(super) Box<Response>);

impl DatabaseError for PostgresDatabaseError {
    fn message(&self) -> &str {
        self.0.message()
    }
}
