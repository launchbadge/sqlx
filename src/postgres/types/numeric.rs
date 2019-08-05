use crate::types::{BigInt, Double, FromSql, Int, Real, SmallInt, ToSql, ToSqlAs};
use byteorder::{BigEndian, ByteOrder};

impl ToSql for i16 {
    type Type = SmallInt;
}

impl ToSqlAs<SmallInt> for i16 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl FromSql<SmallInt> for i16 {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        BigEndian::read_i16(buf)
    }
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

impl FromSql<Int> for i32 {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        BigEndian::read_i32(buf)
    }
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

impl FromSql<BigInt> for i64 {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        BigEndian::read_i64(buf)
    }
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

impl FromSql<BigInt> for f32 {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        f32::from_bits(i32::from_sql(buf) as u32)
    }
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

impl FromSql<Double> for f64 {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        f64::from_bits(i64::from_sql(buf) as u64)
    }
}
