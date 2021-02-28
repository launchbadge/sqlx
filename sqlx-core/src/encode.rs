use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use crate::database::HasOutput;
use crate::{Arguments, Database, Type, TypeEncode};
use std::iter::FromIterator;
use std::ops::BitOr;

/// Type returned from [`Encode::encode`] that indicates if the value encoded is the SQL `null` or not.
pub enum IsNull {
    /// The value is the SQL `null`.
    ///
    /// No data was written to the output buffer.
    ///
    Yes,

    /// The value is not the SQL `null`.
    ///
    /// This does not mean that any data was written to the output buffer. For example,
    /// an empty string has no data, but is not null.
    ///
    No,
}

#[doc(hidden)]
impl BitOr<IsNull> for IsNull {
    type Output = IsNull;

    fn bitor(self, rhs: IsNull) -> Self::Output {
        use IsNull::*;

        match (self, rhs) {
            (No, No) => No,
            (Yes, No) | (No, Yes) | (Yes, Yes) => Yes,
        }
    }
}

/// Useful for encoding arrays
#[doc(hidden)]
impl FromIterator<IsNull> for IsNull {
    fn from_iter<T: IntoIterator<Item = IsNull>>(iter: T) -> Self {
        iter.into_iter().fold(IsNull::No, BitOr::bitor)
    }
}

/// A type that can be encoded into a SQL value.
pub trait Encode<Db: Database>: Send + Sync {
    /// Encode this value into the specified SQL type.
    fn encode(&self, ty: &Db::TypeInfo, out: &mut <Db as HasOutput<'_>>::Output) -> Result;

    /// If this type is a vector, get its length.
    fn vector_len(&self) -> Option<usize> {
        None
    }

    /// If this type is a vector, add its elements as positional arguments to `arguments`.
    ///
    /// Panics if not a vector.
    fn expand_vector<'a>(&'a self, _arguments: &mut Arguments<'a, Db>) {
        panic!("not a vector!")
    }
}

impl<T: Encode<Db>, Db: Database> Encode<Db> for &T {
    #[inline]
    fn encode(&self, ty: &Db::TypeInfo, out: &mut <Db as HasOutput<'_>>::Output) -> Result {
        (*self).encode(ty, out)
    }

    fn vector_len(&self) -> Option<usize> {
        (*self).vector_len()
    }
}

/// Errors which can occur while encoding a SQL value.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    TypeNotCompatible {
        rust_type_name: &'static str,
        sql_type_name: &'static str,
    },

    /// A general error raised while encoding a value.
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
            Self::TypeNotCompatible { rust_type_name, sql_type_name } => {
                write!(
                    f,
                    "Rust type `{}` is not compatible with SQL type `{}`",
                    rust_type_name, sql_type_name
                )
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

/// A specialized result type representing the result of encoding a SQL value.
pub type Result = std::result::Result<IsNull, Error>;
