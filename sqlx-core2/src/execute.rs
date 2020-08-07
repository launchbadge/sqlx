use crate::database::Database;
use crate::arguments::Arguments;

/// A type that may be executed against a SQL database connection.
pub trait Execute<'q, DB: Database>: Send {
    /// Gets the SQL that will be executed.
    fn sql(&self) -> &'q str;

    /// Gets the arguments to be bound against the SQL query.
    fn arguments(&'q mut self) -> Option<&'q Arguments<'q, DB>>;
}

impl<'q, DB: Database> Execute<'q, DB> for &'q str {
    #[inline]
    fn sql(&self) -> &'q str {
        *self
    }

    #[inline]
    fn arguments(&'q mut self) -> Option<&'q Arguments<'q, DB>> {
        None
    }
}
