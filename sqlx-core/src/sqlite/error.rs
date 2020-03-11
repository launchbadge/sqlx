use crate::error::DatabaseError;

pub struct SqliteError;

impl DatabaseError for SqliteError {
    fn message(&self) -> &str {
        todo!()
    }
}

impl_fmt_error!(SqliteError);
