use std::error::Error as StdError;

/// Representing an error that was identified by the client as a result
/// of interacting with the database.
///
/// This can be anything from receiving invalid UTF-8 from the database
/// (where valid UTF-8 is expected) to being asked for interactive
/// authentication (where none is supported).
///
#[allow(clippy::module_name_repetitions)]
pub trait ClientError: 'static + StdError + Send + Sync {}
