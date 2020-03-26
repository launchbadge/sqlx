//! Errorand Result types.

use crate::database::Database;
use crate::types::Type;
use std::any::type_name;
use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::io;

#[allow(unused_macros)]
macro_rules! decode_err {
    ($s:literal, $($args:tt)*) => {
        crate::Error::Decode(format!($s, $($args)*).into())
    };

    ($expr:expr) => {
        crate::Error::decode($expr)
    };
}

/// A specialized `Result` type for SQLx.
pub type Result<T> = std::result::Result<T, Error>;

/// A generic error that represents all the ways a method can fail inside of SQLx.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error communicating with the database.
    Io(io::Error),

    /// Connection URL was malformed.
    UrlParse(url::ParseError),

    /// An error was returned by the database.
    Database(Box<dyn DatabaseError>),

    /// No row was returned during [`query::Map::fetch_one`] or `QueryAs::fetch_one`.
    ///
    /// [`query::Map::fetch_one`]: crate::query::Map::fetch_one
    RowNotFound,

    /// Column was not found by name in a Row (during [`Row::get`]).
    ///
    /// [`Row::get`]: crate::row::Row::get
    ColumnNotFound(Box<str>),

    /// Column index was out of bounds (e.g., asking for column 4 in a 2-column row).
    ColumnIndexOutOfBounds { index: usize, len: usize },

    /// Unexpected or invalid data was encountered. This would indicate that we received
    /// data that we were not expecting or it was in a format we did not understand. This
    /// generally means either there is a programming error in a SQLx driver or
    /// something with the connection or the database database itself is corrupted.
    ///
    /// Context is provided by the included error message.
    Protocol(Box<str>),

    /// A [`Pool::acquire`] timed out due to connections not becoming available or
    /// because another task encountered too many errors while trying to open a new connection.
    ///
    /// [`Pool::acquire`]: crate::pool::Pool::acquire
    PoolTimedOut(Option<Box<dyn StdError + Send + Sync>>),

    /// [`Pool::close`] was called while we were waiting in [`Pool::acquire`].
    ///
    /// [`Pool::acquire`]: crate::pool::Pool::acquire
    /// [`Pool::close`]: crate::pool::Pool::close
    PoolClosed,

    /// An error occurred while attempting to setup TLS.
    /// This should only be returned from an explicit ask for TLS.
    Tls(Box<dyn StdError + Send + Sync>),

    /// An error occurred decoding data received from the database.
    Decode(Box<dyn StdError + Send + Sync>),
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn decode<E>(err: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::Decode(err.into())
    }

    #[allow(dead_code)]
    pub(crate) fn mismatched_types<DB: Database, T>(expected: DB::TypeInfo) -> Self
    where
        T: Type<DB>,
    {
        let ty_name = type_name::<T>();

        return decode_err!(
            "mismatched types; Rust type `{}` (as SQL type {}) is not compatible with SQL type {}",
            ty_name,
            T::type_info(),
            expected
        );
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::UrlParse(error) => Some(error),
            Error::PoolTimedOut(Some(error)) => Some(&**error),
            Error::Decode(error) => Some(&**error),
            Error::Tls(error) => Some(&**error),
            Error::Database(error) => Some(error.as_ref_err()),

            _ => None,
        }
    }
}

impl Display for Error {
    // IntellijRust does not understand that [non_exhaustive] applies only for downstream crates
    // noinspection RsMatchCheck
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{}", error),

            Error::UrlParse(error) => write!(f, "{}", error),

            Error::Decode(error) => write!(f, "{}", error),

            Error::Database(error) => Display::fmt(error, f),

            Error::RowNotFound => f.write_str("found no row when we expected at least one"),

            Error::ColumnNotFound(ref name) => {
                write!(f, "no column found with the name {:?}", name)
            }

            Error::ColumnIndexOutOfBounds { index, len } => write!(
                f,
                "column index out of bounds: there are {} columns but the index is {}",
                len, index
            ),

            Error::Protocol(ref err) => f.write_str(err),

            Error::PoolTimedOut(Some(ref err)) => {
                write!(f, "timed out while waiting for an open connection: {}", err)
            }

            Error::PoolTimedOut(None) => {
                write!(f, "timed out while waiting for an open connection")
            }

            Error::PoolClosed => f.write_str("attempted to acquire a connection on a closed pool"),

            Error::Tls(ref err) => write!(f, "error during TLS upgrade: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<io::ErrorKind> for Error {
    #[inline]
    fn from(err: io::ErrorKind) -> Self {
        Error::Io(err.into())
    }
}

impl From<url::ParseError> for Error {
    #[inline]
    fn from(err: url::ParseError) -> Self {
        Error::UrlParse(err)
    }
}

impl From<ProtocolError<'_>> for Error {
    #[inline]
    fn from(err: ProtocolError) -> Self {
        Error::Protocol(err.args.to_string().into_boxed_str())
    }
}

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
impl From<async_native_tls::Error> for Error {
    #[inline]
    fn from(err: async_native_tls::Error) -> Self {
        Error::Tls(err.into())
    }
}

impl From<TlsError<'_>> for Error {
    #[inline]
    fn from(err: TlsError<'_>) -> Self {
        Error::Tls(err.args.to_string().into())
    }
}

/// An error that was returned by the database.
pub trait DatabaseError: StdError + Send + Sync + 'static {
    /// The primary, human-readable error message.
    fn message(&self) -> &str;

    /// The (SQLSTATE) code for the error.
    fn code(&self) -> Option<&str> {
        None
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        None
    }

    fn column_name(&self) -> Option<&str> {
        None
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }

    #[doc(hidden)]
    fn as_ref_err(&self) -> &(dyn StdError + Send + Sync + 'static);

    #[doc(hidden)]
    fn as_mut_err(&mut self) -> &mut (dyn StdError + Send + Sync + 'static);

    #[doc(hidden)]
    fn into_box_err(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static>;
}

impl dyn DatabaseError {
    /// Downcast this `&dyn DatabaseError` to a specific database error type:
    ///
    /// * [PgError][crate::postgres::PgError] (if the `postgres` feature is active)
    /// * [MySqlError][crate::mysql::MySqlError] (if the `mysql` feature is active)
    /// * [SqliteError][crate::sqlite::SqliteError] (if the `sqlite` feature is active)
    ///
    /// In a generic context you can use the [crate::database::Database::Error] associated type.
    ///
    /// ### Panics
    /// If the type does not match; this is in contrast with [StdError::downcast_ref]
    /// which returns `Option`. This was a deliberate design decision in favor of brevity as in
    /// almost all cases you should know which database error type you're expecting.
    ///
    /// In any other cases, use [Self::try_downcast_ref] instead.
    pub fn downcast_ref<T: DatabaseError>(&self) -> &T {
        self.try_downcast_ref::<T>().unwrap_or_else(|| {
            panic!(
                "downcasting to wrong DatabaseError type; original error: {:?}",
                self
            )
        })
    }

    /// Downcast this `&dyn DatabaseError` to a specific database error type:
    ///
    /// * [PgError][crate::postgres::PgError] (if the `postgres` feature is active)
    /// * [MySqlError][crate::mysql::MySqlError] (if the `mysql` feature is active)
    /// * [SqliteError][crate::sqlite::SqliteError] (if the `sqlite` feature is active)
    ///
    /// In a generic context you can use the [crate::database::Database::Error] associated type.
    ///
    /// Returns `None` if the downcast fails (the types do not match)
    pub fn try_downcast_ref<T: DatabaseError>(&self) -> Option<&T> {
        self.as_ref_err().downcast_ref()
    }

    /// Only meant for internal use so no `try_` variant is currently provided
    #[allow(dead_code)]
    pub(crate) fn downcast_mut<T: DatabaseError>(&mut self) -> &mut T {
        // tried to express this as the following:
        //
        // if let Some(e) = self.as_mut_err().downcast_mut() { return e; }
        //
        // however it didn't like using `self` again in the panic format
        if self.as_ref_err().is::<T>() {
            return self.as_mut_err().downcast_mut().unwrap();
        }

        panic!(
            "downcasting to wrong DatabaseError type; original error: {:?}",
            self
        )
    }

    /// Downcast this `Box<dyn DatabaseError>` to a specific database error type:
    ///
    /// * [PgError][crate::postgres::PgError] (if the `postgres` feature is active)
    /// * [MySqlError][crate::mysql::MySqlError] (if the `mysql` feature is active)
    /// * [SqliteError][crate::sqlite::SqliteError] (if the `sqlite` feature is active)
    ///
    /// In a generic context you can use the [crate::database::Database::Error] associated type.
    ///
    /// ### Panics
    /// If the type does not match; this is in contrast with [std::error::Error::downcast]
    /// which returns `Result`. This was a deliberate design decision in favor of
    /// brevity as in almost all cases you should know which database error type you're expecting.
    ///
    /// In any other cases, use [Self::try_downcast] instead.
    pub fn downcast<T: DatabaseError>(self: Box<Self>) -> Box<T> {
        self.try_downcast().unwrap_or_else(|e| {
            panic!(
                "downcasting to wrong DatabaseError type; original error: {:?}",
                e
            )
        })
    }

    /// Downcast this `Box<dyn DatabaseError>` to a specific database error type:
    ///
    /// * [PgError][crate::postgres::PgError] (if the `postgres` feature is active)
    /// * [MySqlError][crate::mysql::MySqlError] (if the `mysql` feature is active)
    /// * [SqliteError][crate::sqlite::SqliteError] (if the `sqlite` feature is active)
    ///
    /// In a generic context you can use the [crate::database::Database::Error] associated type.
    ///
    /// Returns `Err(self)` if the downcast fails (the types do not match).
    pub fn try_downcast<T: DatabaseError>(
        self: Box<Self>,
    ) -> std::result::Result<Box<T>, Box<Self>> {
        if self.as_ref_err().is::<T>() {
            Ok(self
                .into_box_err()
                .downcast()
                .expect("type mismatch between DatabaseError::as_ref_err() and into_box_err()"))
        } else {
            Err(self)
        }
    }
}

/// Used by the `protocol_error!()` macro for a lazily evaluated conversion to
/// `crate::Error::Protocol` so we can use the macro with `.ok_or()` without Clippy complaining.
pub(crate) struct ProtocolError<'a> {
    pub args: fmt::Arguments<'a>,
}

#[allow(unused_macros)]
macro_rules! protocol_err (
    ($($args:tt)*) => {
        $crate::error::ProtocolError { args: format_args!($($args)*) }
    }
);

pub(crate) struct TlsError<'a> {
    pub args: fmt::Arguments<'a>,
}

#[allow(unused_macros)]
macro_rules! tls_err {
    ($($args:tt)*) => { crate::error::TlsError { args: format_args!($($args)*)} };
}

/// An unexpected `NULL` was encountered during decoding.
///
/// Returned from `Row::get` if the value from the database is `NULL`
/// and you are not decoding into an `Option`.
#[derive(Debug, Clone, Copy)]
pub struct UnexpectedNullError;

impl Display for UnexpectedNullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unexpected null; try decoding as an `Option`")
    }
}

impl StdError for UnexpectedNullError {}
