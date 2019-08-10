use crate::backend::Backend;

/// Information about how a backend stores metadata about
/// given SQL types.
pub trait TypeMetadata {
    /// The actual type used to represent metadata.
    type TypeMetadata;
}

/// Indicates that a SQL type exists for a backend and defines
/// useful metadata for the backend.
pub trait HasSqlType<A>: TypeMetadata {
    fn metadata() -> Self::TypeMetadata;
}

/// Defines the canonical SQL that the implementing Rust type represents.
/// This trait is used to map Rust types to SQL types when the explicit mapping is missing.
pub trait AsSqlType<DB: Backend>
where
    DB: HasSqlType<Self::SqlType>,
{
    type SqlType;
}

impl<T, DB> AsSqlType<DB> for Option<T>
where
    DB: Backend + HasSqlType<<T as AsSqlType<DB>>::SqlType>,
    T: AsSqlType<DB>,
{
    type SqlType = T::SqlType;
}

// Character types
// All character types (VARCHAR, CHAR, TEXT, etc.) are represented equivalently in binary and all fold
// to this `Text` type.

pub struct Text;

// Numeric types

// i16
pub struct SmallInt;

// i32
pub struct Int;

// i64
pub struct BigInt;

// decimal?
// TODO pub struct Decimal;

// f32
pub struct Real;

// f64
pub struct Double;
