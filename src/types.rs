use crate::backend::Backend;

// TODO: Does [AsSql] need to be generic over back-end ?

pub trait SqlType<B>
where
    B: Backend,
{
    // FIXME: This should be a const fn
    fn metadata() -> B::TypeMetadata;
}

/// Defines the canonical SQL that the implementing Rust type represents.
/// This trait is used to map Rust types to SQL types when the explicit mapping is missing.
pub trait AsSql<B>
where
    B: Backend,
{
    /// SQL type that should be inferred from the implementing Rust type.
    type Type: SqlType<B>;
}

impl<B, T> AsSql<B> for Option<T>
where
    B: Backend,
    T: AsSql<B>,
{
    type Type = T::Type;
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
