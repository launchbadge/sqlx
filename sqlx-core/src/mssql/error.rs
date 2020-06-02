use crate::error::DatabaseError;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

/// An error returned from the MSSQL database.
pub struct MsSqlDatabaseError {}

impl Debug for MsSqlDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl Display for MsSqlDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl Error for MsSqlDatabaseError {}

impl DatabaseError for MsSqlDatabaseError {
    #[inline]
    fn message(&self) -> &str {
        unimplemented!()
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }
}
