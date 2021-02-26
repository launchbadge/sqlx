use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::str::Utf8Error;

use crate::database::HasRawValue;
use crate::{Database, Type, TypeInfo};

/// A type that can be decoded from a SQL value.
pub trait Decode<'r, Db: Database>: Sized + Send + Sync {
    /// Decode the SQL value into the target type.
    fn decode(value: <Db as HasRawValue<'r>>::RawValue) -> Result<Self>;
}

/// A type that can be decoded from a SQL value, without borrowing any data
/// from the row.
#[allow(clippy::module_name_repetitions)]
pub trait DecodeOwned<Db: Database>: for<'r> Decode<'r, Db> {}

impl<T, Db: Database> DecodeOwned<Db> for T where T: for<'r> Decode<'r, Db> {}

/// Errors which can occur while decoding a SQL value.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    TypeNotCompatible {
        rust_type_name: &'static str,
        sql_type_name: &'static str,
    },

    /// An unexpected SQL `NULL` was encountered during decoding.
    ///
    /// To decode potentially `NULL` values, wrap the target type in `Option`.
    ///
    UnexpectedNull,

    /// Attempted to decode non-UTF-8 data into a Rust `str`.
    NotUtf8(Utf8Error),

    /// A general error raised while decoding a value.
    Custom(Box<dyn StdError + Send + Sync>),
}

impl Error {
    #[doc(hidden)]
    pub fn msg<D: Display>(msg: D) -> Self {
        Self::Custom(msg.to_string().into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedNull => f.write_str("unexpected null; try decoding as an `Option`"),

            Self::TypeNotCompatible { rust_type_name, sql_type_name } => {
                write!(
                    f,
                    "Rust type `{}` is not compatible with SQL type `{}`",
                    rust_type_name, sql_type_name
                )
            }

            Self::NotUtf8(error) => {
                write!(f, "{}", error)
            }

            Self::Custom(error) => {
                write!(f, "{}", error)
            }
        }
    }
}

// noinspection DuplicatedCode
impl<E: StdError + Send + Sync + 'static> From<E> for Error {
    fn from(error: E) -> Self {
        Self::Custom(Box::new(error))
    }
}

/// A specialized result type representing the result of decoding a SQL value.
pub type Result<T> = std::result::Result<T, Error>;
