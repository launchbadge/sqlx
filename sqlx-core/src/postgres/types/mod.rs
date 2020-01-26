use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::ops::Deref;
use std::sync::Arc;

use crate::postgres::protocol::TypeId;
use crate::types::TypeInfo;

mod bool;
mod bytes;
mod float;
mod int;
mod str;

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
        Self { id, name: Some(name.into()) }
    }

    /// Create a `PgTypeInfo` from a type's object identifier.
    ///
    /// The object identifier of a type can be queried with
    /// `SELECT oid FROM pg_type WHERE typname = <name>;`
    pub fn with_oid(oid: u32) -> Self {
        Self { id: TypeId(oid), name: None }
    }

    #[doc(hidden)]
    pub fn type_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<UNKNOWN>")
    }
}

impl Display for PgTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{} (type OID {})", *name, self.id.0)
        } else {
            write!(f, "{}", self.id.0)
        }
    }
}

impl TypeInfo for PgTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        // TODO: 99% of postgres types are direct equality for [compatible]; when we add something that isn't (e.g, JSON/JSONB), fix this here
        self.id.0 == other.id.0
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