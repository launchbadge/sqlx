use crate::error::DatabaseError;
use crate::postgres::protocol::Response;

pub struct PgError(pub(super) Box<Response>);

impl DatabaseError for PgError {
    fn message(&self) -> &str {
        &self.0.message
    }

    fn details(&self) -> Option<&str> {
        self.0.detail.as_ref().map(|s| &**s)
    }

    fn hint(&self) -> Option<&str> {
        self.0.hint.as_ref().map(|s| &**s)
    }

    fn table_name(&self) -> Option<&str> {
        self.0.table.as_ref().map(|s| &**s)
    }

    fn column_name(&self) -> Option<&str> {
        self.0.column.as_ref().map(|s| &**s)
    }

    fn constraint_name(&self) -> Option<&str> {
        self.0.constraint.as_ref().map(|s| &**s)
    }
}

impl_fmt_error!(PgError);
