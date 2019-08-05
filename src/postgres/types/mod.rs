use crate::types::{SqlType, ToSql, ToSqlAs};

// TODO: Generalize by Backend and move common types to crate [sqlx::types]

// Character
// https://www.postgresql.org/docs/devel/datatype-character.html

pub struct Text;

impl SqlType for Text {
    const OID: u32 = 25;
}

impl ToSql for &'_ str {
    type Type = Text;
}

impl ToSqlAs<Text> for &'_ str {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

// Numeric
// https://www.postgresql.org/docs/devel/datatype-numeric.html

// i16
pub struct SmallInt;

impl SqlType for SmallInt {
    const OID: u32 = 21;
}

impl ToSql for i16 {
    type Type = SmallInt;
}

impl ToSqlAs<SmallInt> for i16 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

// i32
pub struct Int;

impl SqlType for Int {
    const OID: u32 = 23;
}

impl ToSql for i32 {
    type Type = Int;
}

impl ToSqlAs<Int> for i32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

// i64
pub struct BigInt;

impl SqlType for BigInt {
    const OID: u32 = 20;
}

impl ToSql for i64 {
    type Type = BigInt;
}

impl ToSqlAs<BigInt> for i64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

// decimal?
// TODO pub struct Decimal;

// f32
pub struct Real;

impl SqlType for Real {
    const OID: u32 = 700;
}

impl ToSql for f32 {
    type Type = Real;
}

impl ToSqlAs<Real> for f32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        (self.to_bits() as i32).to_sql(buf);
    }
}

// f64
pub struct Double;

impl SqlType for Double {
    const OID: u32 = 701;
}

impl ToSql for f64 {
    type Type = Double;
}

impl ToSqlAs<Double> for f64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        (self.to_bits() as i64).to_sql(buf);
    }
}
