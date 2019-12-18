use crate::{error::DatabaseError, mysql::protocol::ErrorCode};

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mysql returned an error: {}; {}",
            self.code, self.message
        )
    }
}
