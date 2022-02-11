use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Represents ltree specific errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// LTree labels can only contain [A-Za-z0-9_]
    #[error("ltree label cotains invalid characters")]
    InvalidLtreeLabel,

    /// LTree version not supported
    #[error("ltree version not supported")]
    InvalidLtreeVersion,
}

/// Represents an postgres ltree. Not that this is an EXTENSION!
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PgLTree {
    labels: Vec<String>,
}

impl PgLTree {
    /// creates default/empty ltree
    pub fn new() -> Self {
        Self::default()
    }

    /// creates ltree from a [Vec<String>] without checking labels
    pub fn new_unchecked(labels: Vec<String>) -> Self {
        Self { labels }
    }

    /// creates ltree from an iterator with checking labels
    pub fn from_iter(
        labels: impl IntoIterator<Item = String>,
    ) -> Result<Self, Error> {
        let mut ltree = Self::default();
        for label in labels.into_iter() {
            ltree.push(label)?;
        }
        Ok(ltree)
    }

    /// push a label to ltree
    pub fn push(&mut self, label: String) -> Result<(), Error> {
        if label
            .bytes()
            .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == b'_')
        {
            self.labels.push(label);
            Ok(())
        } else {
            Err(Error::InvalidLtreeLabel)
        }
    }

    /// pop a label from ltree
    pub fn pop(&mut self) -> Option<String> {
        self.labels.pop()
    }
}

impl IntoIterator for PgLTree {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.labels.into_iter()
    }
}

impl FromStr for PgLTree {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(Self {
            labels: s.split('.').map(|s| s.to_owned()).collect(),
        })
    }
}

impl Display for PgLTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.labels.iter();
        if let Some(label) = iter.next() {
            write!(f, "{}", label)?;
            while let Some(label) = iter.next() {
                write!(f, ".{}", label)?;
            }
        }
        Ok(())
    }
}

impl Type<Postgres> for PgLTree {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("ltree")
    }
}

impl Encode<'_, Postgres> for PgLTree {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(1i8.to_le_bytes());
        buf.extend_display(self);

        IsNull::No
    }
}

impl<'r> Decode<'r, Postgres> for PgLTree {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let bytes = value.as_bytes()?;
                let version = i8::from_le_bytes([bytes[0]; 1]);
                if version != 1 {
                    return Err(Box::new(Error::InvalidLtreeVersion));
                }
                Ok(Self::from_str(std::str::from_utf8(&bytes[1..])?)?)
            }
            PgValueFormat::Text => Ok(Self::from_str(value.as_str()?)?),
        }
    }
}
