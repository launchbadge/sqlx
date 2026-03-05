use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display, Formatter};

pub(crate) use sqlx_core::error::*;

/// An error returned from the MSSQL database.
pub struct MssqlDatabaseError {
    pub(crate) number: u32,
    pub(crate) state: u8,
    pub(crate) class: u8,
    pub(crate) message: String,
    pub(crate) server: Option<String>,
    pub(crate) procedure: Option<String>,
}

impl MssqlDatabaseError {
    /// The error number returned by SQL Server.
    pub fn number(&self) -> u32 {
        self.number
    }

    /// The error state.
    pub fn state(&self) -> u8 {
        self.state
    }

    /// The severity class of the error.
    pub fn class(&self) -> u8 {
        self.class
    }

    /// The server name that generated the error, if available.
    pub fn server(&self) -> Option<&str> {
        self.server.as_deref()
    }

    /// The stored procedure name, if applicable.
    pub fn procedure(&self) -> Option<&str> {
        self.procedure.as_deref()
    }
}

impl Debug for MssqlDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MssqlDatabaseError")
            .field("number", &self.number)
            .field("state", &self.state)
            .field("class", &self.class)
            .field("message", &self.message)
            .finish()
    }
}

impl Display for MssqlDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(number {}, state {}): {}", self.number, self.state, self.message)
    }
}

impl StdError for MssqlDatabaseError {}

impl DatabaseError for MssqlDatabaseError {
    #[inline]
    fn message(&self) -> &str {
        &self.message
    }

    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Owned(self.number.to_string()))
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }

    fn kind(&self) -> ErrorKind {
        match self.number {
            // Cannot insert duplicate key
            2601 | 2627 => ErrorKind::UniqueViolation,
            // Foreign key constraint violation
            547 => ErrorKind::ForeignKeyViolation,
            // Cannot insert NULL
            515 => ErrorKind::NotNullViolation,
            // Check constraint violation
            2628 => ErrorKind::CheckViolation,
            _ => ErrorKind::Other,
        }
    }
}

/// Convert a tiberius error into an sqlx Error.
pub(crate) fn tiberius_err(err: tiberius::error::Error) -> Error {
    match err {
        tiberius::error::Error::Server(token_error) => {
            Error::Database(Box::new(MssqlDatabaseError {
                number: token_error.code(),
                state: token_error.state(),
                class: token_error.class(),
                message: token_error.message().to_string(),
                server: {
                    let s = token_error.server();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                },
                procedure: {
                    let s = token_error.procedure();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                },
            }))
        }
        tiberius::error::Error::Io { kind, message } => {
            Error::Io(std::io::Error::new(kind, message))
        }
        other => Error::Protocol(other.to_string()),
    }
}
