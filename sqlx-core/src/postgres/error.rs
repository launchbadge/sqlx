use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::error::DatabaseError;
use crate::postgres::protocol::Response;

#[derive(Debug)]
pub struct PgError(pub(super) Response);

impl DatabaseError for PgError {
    fn message(&self) -> &str {
        &self.0.message
    }

    fn code(&self) -> Option<&str> {
        Some(&self.0.code)
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

    fn as_ref_err(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_mut_err(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_box_err(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }
}

impl Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.message())
    }
}

impl StdError for PgError {}

impl From<PgError> for crate::Error {
    fn from(err: PgError) -> Self {
        crate::Error::Database(Box::new(err))
    }
}

#[test]
fn test_error_downcasting() {
    use super::protocol::Severity;

    let error = PgError(Response {
        severity: Severity::Panic,
        code: "".into(),
        message: "".into(),
        detail: None,
        hint: None,
        position: None,
        internal_position: None,
        internal_query: None,
        where_: None,
        schema: None,
        table: None,
        column: None,
        data_type: None,
        constraint: None,
        file: None,
        line: None,
        routine: None,
    });

    let error = crate::Error::from(error);

    let db_err = match error {
        crate::Error::Database(db_err) => db_err,
        e => panic!("expected Error::Database, got {:?}", e),
    };

    assert_eq!(db_err.downcast_ref::<PgError>().0.severity, Severity::Panic);
    assert_eq!(db_err.downcast::<PgError>().0.severity, Severity::Panic);
}
