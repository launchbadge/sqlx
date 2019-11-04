/// Information about how a backend stores metadata about
/// given SQL types.
pub trait TypeMetadata {
    /// The actual type used to represent metadata.
    type TypeMetadata;
}

/// Indicates that a SQL type exists for a backend and defines
/// useful metadata for the backend.
pub trait HasSqlType<A: ?Sized>: TypeMetadata {
    fn metadata() -> Self::TypeMetadata;
}

impl<A: ?Sized, DB> HasSqlType<&'_ A> for DB
    where DB: HasSqlType<A>
{
    fn metadata() -> Self::TypeMetadata {
        <DB as HasSqlType<A>>::metadata()
    }
}

impl<A, DB> HasSqlType<Option<A>> for DB
    where DB: HasSqlType<A>
{
    fn metadata() -> Self::TypeMetadata {
        <DB as HasSqlType<A>>::metadata()
    }
}
