#[cfg(feature = "uuid")]
pub use uuid::Uuid;

use std::fmt::Display;

/// Information about how a backend stores metadata about
/// given SQL types.
pub trait HasTypeMetadata {
    /// The actual type used to represent metadata.
    type TypeMetadata: TypeMetadata<Self::TypeId>;

    /// The Rust type of type identifiers in `DESCRIBE` responses for the SQL backend.
    type TypeId: Eq + Display;
}

pub trait TypeMetadata<TypeId: Eq> {
    /// Return `true` if the given type ID is contained in this metadata.
    ///
    /// What this means depends on the backend:
    ///
    /// * For Postgres, this should return true if the type ID or array type ID matches.
    /// * For MySQL (and likely all other backends) this should just compare the type IDs.
    fn type_id_eq(&self, other: &TypeId) -> bool;
}

/// Indicates that a SQL type exists for a backend and defines
/// useful metadata for the backend.
pub trait HasSqlType<A: ?Sized>: HasTypeMetadata {
    fn metadata() -> Self::TypeMetadata;
}

impl<A: ?Sized, DB> HasSqlType<&'_ A> for DB
where
    DB: HasSqlType<A>,
{
    fn metadata() -> Self::TypeMetadata {
        <DB as HasSqlType<A>>::metadata()
    }
}

impl<A, DB> HasSqlType<Option<A>> for DB
where
    DB: HasSqlType<A>,
{
    fn metadata() -> Self::TypeMetadata {
        <DB as HasSqlType<A>>::metadata()
    }
}
