/// Information about how a backend stores metadata about
/// given SQL types.
pub trait HasTypeMetadata {
    /// The actual type used to represent metadata.
    type TypeMetadata: TypeMetadata<TypeId = Self::TypeId>;

    /// The Rust type of the type ID for the backend.
    type TypeId: Eq;

    /// UNSTABLE: for internal use only
    #[doc(hidden)]
    fn param_type_for_id(id: &Self::TypeId) -> Option<&'static str>;

    /// UNSTABLE: for internal use only
    #[doc(hidden)]
    fn return_type_for_id(id: &Self::TypeId) -> Option<&'static str>;
}

pub trait TypeMetadata {
    type TypeId: Eq;

    fn type_id(&self) -> &Self::TypeId;
    fn type_id_eq(&self, id: &Self::TypeId) -> bool {
        self.type_id() == id
    }
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
