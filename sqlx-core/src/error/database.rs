use std::error::Error as StdError;

/// `DatabaseError` is a trait representing an error that was returned from
/// the database.
///
/// Provides abstract access to information returned from the database about
/// the error.
///
#[allow(clippy::module_name_repetitions)]
pub trait DatabaseError: 'static + StdError + Send + Sync {
    /// Returns the primary, human-readable error message.
    fn message(&self) -> &str;
}
