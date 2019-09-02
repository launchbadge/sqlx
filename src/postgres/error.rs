use super::protocol::Response;
use crate::error::DbError;

#[derive(Debug)]
pub struct PostgresError(pub(super) Box<Response>);

impl DbError for PostgresError {
    fn message(&self) -> &str {
        self.0.message()
    }
}
