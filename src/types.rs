use crate::backend::Backend;

pub use crate::postgres::types::*;

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

// impl SqlType for Text {
//     const OID: u32 = 25;
// }

// Numeric types

// i16
pub struct SmallInt;

// impl SqlType for SmallInt {
//     const OID: u32 = 21;
// }

// i32
pub struct Int;

// impl SqlType for Int {
//     const OID: u32 = 23;
// }

// i64
pub struct BigInt;

// impl SqlType for BigInt {
//     const OID: u32 = 20;
// }

// decimal?
// TODO pub struct Decimal;

// f32
pub struct Real;

// impl SqlType for Real {
//     const OID: u32 = 700;
// }

// f64
pub struct Double;

// impl SqlType for Double {
//     const OID: u32 = 701;
// }
