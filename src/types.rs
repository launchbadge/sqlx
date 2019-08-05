pub use crate::postgres::types::*;

// TODO: Better name for ToSql/ToSqlAs. ToSqlAs is the _conversion_ trait.
//       ToSql is type fallback for Rust/SQL (e.g., what is the probable SQL type for this Rust type)

// TODO: Make generic over backend

pub trait SqlType {
    // FIXME: This is a postgres thing
    const OID: u32;
}

pub trait ToSql {
    /// SQL type that should be inferred from the implementing Rust type.
    type Type: SqlType;
}

pub trait ToSqlAs<T: SqlType>: ToSql {
    fn to_sql(self, buf: &mut Vec<u8>);
}

pub trait FromSql<T: SqlType>: ToSql {
    // TODO: Errors?
    fn from_sql(buf: &[u8]) -> Self;
}

// Character types
// All character types (VARCHAR, CHAR, TEXT, etc.) are represented equivalently in binary and all fold
// to this `Text` type.

pub struct Text;

impl SqlType for Text {
    // FIXME: This is postgres-specific
    const OID: u32 = 25;
}

// Numeric types

// i16
pub struct SmallInt;

impl SqlType for SmallInt {
    const OID: u32 = 21;
}

// i32
pub struct Int;

impl SqlType for Int {
    const OID: u32 = 23;
}

// i64
pub struct BigInt;

impl SqlType for BigInt {
    const OID: u32 = 20;
}

// decimal?
// TODO pub struct Decimal;

// f32
pub struct Real;

impl SqlType for Real {
    const OID: u32 = 700;
}

// f64
pub struct Double;

impl SqlType for Double {
    const OID: u32 = 701;
}
