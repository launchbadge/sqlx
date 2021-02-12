use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use crate::database::HasOutput;
use crate::Database;

/// A type that can be encoded into a SQL value.
pub trait Encode<Db: Database>: Send + Sync {
    /// Encode this value into a SQL value.
    fn encode(&self, ty: &Db::TypeInfo, out: &mut <Db as HasOutput<'_>>::Output) -> Result<()>;

    #[doc(hidden)]
    #[inline]
    fn __type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Encode<Db>, Db: Database> Encode<Db> for &T {
    #[inline]
    fn encode(&self, ty: &Db::TypeInfo, out: &mut <Db as HasOutput<'_>>::Output) -> Result<()> {
        (*self).encode(ty, out)
    }
}

/// Errors which can occur while encoding a SQL value.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A general error raised while encoding a value.
    Custom(Box<dyn StdError + Send + Sync>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
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
pub type Result<T> = std::result::Result<T, Error>;
