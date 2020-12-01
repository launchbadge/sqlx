use crate::error::DatabaseError;

use rusoto_core::RusotoError;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

/// An error returned from the Aurora database.
#[derive(Debug)]
pub struct AuroraDatabaseError<E: Error>(pub(crate) RusotoError<E>);

impl<T: Error + 'static> Display for AuroraDatabaseError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl<T: Error + 'static> Error for AuroraDatabaseError<T> {}

impl<T: Error + Send + Sync + 'static> DatabaseError for AuroraDatabaseError<T> {
    fn message(&self) -> &str {
        self.0.description()
    }

    fn code(&self) -> Option<Cow<'_, str>> {
        None
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
