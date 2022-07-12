use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

/// Represents ltree specific errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PgLTreeParseError {
    /// LTree labels can only contain [A-Za-z0-9_]
    #[error("ltree label contains invalid characters")]
    InvalidLtreeLabel,

    /// LTree version not supported
    #[error("ltree version not supported")]
    InvalidLtreeVersion,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PgLTreeLabel(String);

impl PgLTreeLabel {
    pub fn new<S>(label: S) -> Result<Self, PgLTreeParseError>
    where
        String: From<S>,
    {
        let label = String::from(label);
        if label.len() <= 256
            && label
                .bytes()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == b'_')
        {
            Ok(Self(label))
        } else {
            Err(PgLTreeParseError::InvalidLtreeLabel)
        }
    }
}

impl Deref for PgLTreeLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromStr for PgLTreeLabel {
    type Err = PgLTreeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PgLTreeLabel::new(s)
    }
}

impl Display for PgLTreeLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Container for a Label Tree (`ltree`) in Postgres.
///
/// See https://www.postgresql.org/docs/current/ltree.html
///
/// ### Note: Requires Postgres 13+
///
/// This integration requires that the `ltree` type support the binary format in the Postgres
/// wire protocol, which only became available in Postgres 13.
/// ([Postgres 13.0 Release Notes, Additional Modules][https://www.postgresql.org/docs/13/release-13.html#id-1.11.6.11.5.14])
///
/// Ideally, SQLx's Postgres driver should support falling back to text format for types
/// which don't have `typsend` and `typrecv` entries in `pg_type`, but that work still needs
/// to be done.
///
/// ### Note: Extension Required
/// The `ltree` extension is not enabled by default in Postgres. You will need to do so explicitly:
///
/// ```ignore
/// CREATE EXTENSION IF NOT EXISTS "ltree";
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PgLTree {
    labels: Vec<PgLTreeLabel>,
}

impl PgLTree {
    /// creates default/empty ltree
    pub fn new() -> Self {
        Self::default()
    }

    /// creates ltree from a [Vec<PgLTreeLabel>]
    pub fn from(labels: Vec<PgLTreeLabel>) -> Self {
        Self { labels }
    }

    /// creates ltree from an iterator with checking labels
    pub fn from_iter<I, S>(labels: I) -> Result<Self, PgLTreeParseError>
    where
        String: From<S>,
        I: IntoIterator<Item = S>,
    {
        let mut ltree = Self::default();
        for label in labels {
            ltree.push(PgLTreeLabel::new(label)?);
        }
        Ok(ltree)
    }

    /// push a label to ltree
    pub fn push(&mut self, label: PgLTreeLabel) {
        self.labels.push(label);
    }

    /// pop a label from ltree
    pub fn pop(&mut self) -> Option<PgLTreeLabel> {
        self.labels.pop()
    }
}

impl IntoIterator for PgLTree {
    type Item = PgLTreeLabel;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.labels.into_iter()
    }
}

impl FromStr for PgLTree {
    type Err = PgLTreeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            labels: s
                .split('.')
                .map(|s| PgLTreeLabel::new(s))
                .collect::<Result<Vec<_>, Self::Err>>()?,
        })
    }
}

impl Display for PgLTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.labels.iter();
        if let Some(label) = iter.next() {
            write!(f, "{}", label)?;
            for label in iter {
                write!(f, ".{}", label)?;
            }
        }
        Ok(())
    }
}

impl Deref for PgLTree {
    type Target = [PgLTreeLabel];

    fn deref(&self) -> &Self::Target {
        &self.labels
    }
}

impl Type<Postgres> for PgLTree {
    fn type_info() -> PgTypeInfo {
        // Since `ltree` is enabled by an extension, it does not have a stable OID.
        PgTypeInfo::with_name("ltree")
    }
}

impl PgHasArrayType for PgLTree {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_ltree")
    }
}

impl Encode<'_, Postgres> for PgLTree {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(1i8.to_le_bytes());
        write!(buf, "{}", self)
            .expect("Display implementation panicked while writing to PgArgumentBuffer");

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
                    return Err(Box::new(PgLTreeParseError::InvalidLtreeVersion));
                }
                Ok(Self::from_str(std::str::from_utf8(&bytes[1..])?)?)
            }
            PgValueFormat::Text => Ok(Self::from_str(value.as_str()?)?),
        }
    }
}
