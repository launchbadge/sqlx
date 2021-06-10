use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use sqlx_core::{DatabaseError, Error};

use crate::protocol::ErrPacket;

/// An error returned from the MySQL database.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct MySqlDatabaseError(pub(crate) ErrPacket);

impl MySqlDatabaseError {
    /// Returns a human-readable error message.
    pub fn message(&self) -> &str {
        &*self.0.error_message
    }

    /// Returns the error code.
    ///
    /// All possible error codes should be documented in
    /// the [Server Error Message Reference]. Each code refers to a
    /// unique error messasge.
    ///
    /// [Server Error Message Reference]: https://dev.mysql.com/doc/mysql-errors/8.0/en/server-error-reference.html
    ///
    pub const fn code(&self) -> u16 {
        self.0.error_code
    }

    /// Return the [SQLSTATE] error code.
    ///
    /// The error code consists of 5 characters with `"00000"`
    /// meaning "no error". [SQLSTATE] values are defined by the SQL standard
    /// and should be consistent across databases.
    ///
    /// [SQLSTATE]: https://en.wikipedia.org/wiki/SQLSTATE
    ///
    pub fn sql_state(&self) -> &str {
        self.0.sql_state.as_deref().unwrap_or_default()
    }
}

impl DatabaseError for MySqlDatabaseError {
    fn message(&self) -> &str {
        &self.0.error_message
    }

    #[inline(always)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    #[inline(always)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    #[inline(always)]
    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }
}

impl Display for MySqlDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0.sql_state {
            Some(state) => write!(f, "{} ({}): {}", self.0.error_code, state, self.message()),
            None => write!(f, "{}: {}", self.0.error_code, self.message()),
        }
    }
}

impl StdError for MySqlDatabaseError {}

impl From<MySqlDatabaseError> for Error {
    fn from(err: MySqlDatabaseError) -> Self {
        Self::database(err)
    }
}
