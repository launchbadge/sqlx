mod array;
mod bool;
mod bytes;
mod float;
mod int;
mod str;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "uuid")]
mod uuid;

use std::fmt::{self, Debug, Display};

use crate::postgres::protocol::TypeId;
use crate::types::TypeInfo;

#[derive(Debug, Clone)]
pub struct PgTypeInfo {
    pub(crate) id: TypeId,
}

impl PgTypeInfo {
    pub(crate) fn new(id: TypeId) -> Self {
        Self { id }
    }

    /// Create a `PgTypeInfo` from a type's object identifier.
    ///
    /// The object identifier of a type can be queried with
    /// `SELECT oid FROM pg_type WHERE typname = <name>;`
    pub fn with_oid(oid: u32) -> Self {
        Self { id: TypeId(oid) }
    }
}

impl Display for PgTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Should we attempt to render the type *name* here?
        write!(f, "{}", self.id.0)
    }
}

impl TypeInfo for PgTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        // TODO: 99% of postgres types are direct equality for [compatible]; when we add something that isn't (e.g, JSON/JSONB), fix this here
        self.id.0 == other.id.0
    }
}
