use crate::{error::DatabaseError, mariadb::protocol::ErrorCode};

use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: Box<str>,
}

impl DatabaseError for Error {
    fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MariaDB returned an error: {}",)
    }
}
