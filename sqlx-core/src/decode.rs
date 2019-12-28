//! Types and traits for decoding values from the database.

use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::database::Database;
use crate::types::HasSqlType;

pub enum DecodeError {
    /// An unexpected `NULL` was encountered while decoding.
    UnexpectedNull,

    Message(Box<dyn Display + Send + Sync>),

    Other(Box<dyn StdError + Send + Sync>),
}

/// Decode a single value from the database.
pub trait Decode<DB>: Sized
where
    DB: Database + ?Sized,
{
    fn decode(raw: &[u8]) -> Result<Self, DecodeError>;

    /// Creates a new value of this type from a `NULL` SQL value.
    ///
    /// The default implementation returns [DecodeError::UnexpectedNull].
    fn decode_null() -> Result<Self, DecodeError> {
        Err(DecodeError::UnexpectedNull)
    }

    fn decode_nullable(raw: Option<&[u8]>) -> Result<Self, DecodeError> {
        if let Some(raw) = raw {
            Self::decode(raw)
        } else {
            Self::decode_null()
        }
    }
}

impl<T, DB> Decode<DB> for Option<T>
where
    DB: Database + HasSqlType<T>,
    T: Decode<DB>,
{
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        T::decode(buf).map(Some)
    }

    fn decode_null() -> Result<Self, DecodeError> {
        Ok(None)
    }
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("DecodeError(")?;

        match self {
            DecodeError::UnexpectedNull => write!(f, "unexpected null for non-null column")?,
            DecodeError::Message(err) => write!(f, "{}", err)?,
            DecodeError::Other(err) => write!(f, "{:?}", err)?,
        }

        f.write_str(")")
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError::UnexpectedNull => f.write_str("unexpected null for non-null column"),
            DecodeError::Message(err) => write!(f, "{}", err),
            DecodeError::Other(err) => write!(f, "{}", err),
        }
    }
}

impl<E> From<E> for DecodeError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(err: E) -> DecodeError {
        DecodeError::Other(Box::new(err))
    }
}
