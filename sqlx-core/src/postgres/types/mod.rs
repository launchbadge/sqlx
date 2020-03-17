use std::fmt::{self, Debug, Display};
use std::ops::Deref;
use std::sync::Arc;

use crate::decode::Decode;
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgValue, Postgres};
use crate::types::TypeInfo;

mod bool;
mod bytes;
mod float;
mod int;
mod numeric;
mod record;
mod str;

pub use self::numeric::{PgNumeric, PgNumericSign};
pub use self::record::{PgRecordDecoder, PgRecordEncoder};

#[cfg(feature = "bigdecimal_bigint")]
mod bigdecimal;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "uuid")]
mod uuid;

#[derive(Debug, Clone)]
pub struct PgTypeInfo {
    pub(crate) id: TypeId,
    pub(crate) name: Option<SharedStr>,
}

impl PgTypeInfo {
    pub(crate) fn new(id: TypeId, name: impl Into<SharedStr>) -> Self {
        Self {
            id,
            name: Some(name.into()),
        }
    }

    /// Create a `PgTypeInfo` from a type's object identifier.
    ///
    /// The object identifier of a type can be queried with
    /// `SELECT oid FROM pg_type WHERE typname = <name>;`
    pub fn with_oid(oid: u32) -> Self {
        Self {
            id: TypeId(oid),
            name: None,
        }
    }

    #[doc(hidden)]
    pub fn type_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<UNKNOWN>")
    }

    #[doc(hidden)]
    pub fn type_feature_gate(&self) -> Option<&'static str> {
        match self.id {
            TypeId::DATE | TypeId::TIME | TypeId::TIMESTAMP | TypeId::TIMESTAMPTZ => Some("chrono"),
            TypeId::UUID => Some("uuid"),
            _ => None,
        }
    }

    pub fn oid(&self) -> u32 {
        self.id.0
    }
}

impl Display for PgTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{}", *name)
        } else {
            write!(f, "OID {}", self.id.0)
        }
    }
}

impl TypeInfo for PgTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        // TODO: 99% of postgres types are direct equality for [compatible]; when we add something that isn't (e.g, JSON/JSONB), fix this here
        self.id.0 == other.id.0
    }
}

impl<'de, T> Decode<'de, Postgres> for Option<T>
where
    T: Decode<'de, Postgres>,
{
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        value
            .map(|value| <T as Decode<Postgres>>::decode(Some(value)))
            .transpose()
    }
}

/// Copy of `Cow` but for strings; clones guaranteed to be cheap.
#[derive(Clone, Debug)]
pub(crate) enum SharedStr {
    Static(&'static str),
    Arc(Arc<str>),
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            SharedStr::Static(s) => s,
            SharedStr::Arc(s) => s,
        }
    }
}

impl<'a> From<&'a SharedStr> for SharedStr {
    fn from(s: &'a SharedStr) -> Self {
        s.clone()
    }
}

impl From<&'static str> for SharedStr {
    fn from(s: &'static str) -> Self {
        SharedStr::Static(s)
    }
}

impl From<String> for SharedStr {
    #[inline]
    fn from(s: String) -> Self {
        SharedStr::Arc(s.into())
    }
}

impl fmt::Display for SharedStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self)
    }
}
