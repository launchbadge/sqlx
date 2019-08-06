use super::TypeMetadata;
use crate::{
    deserialize::FromSql,
    postgres::Postgres,
    serialize::{IsNull, ToSql},
    types::{AsSql, BigInt, Double, Int, Real, SmallInt, SqlType},
};
use byteorder::{BigEndian, ByteOrder};

impl SqlType<Postgres> for SmallInt {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 21,
            array_oid: 1005,
        }
    }
}

impl AsSql<Postgres> for i16 {
    type Type = SmallInt;
}

impl ToSql<Postgres, SmallInt> for i16 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Postgres, SmallInt> for i16 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        BigEndian::read_i16(buf.unwrap())
    }
}

impl SqlType<Postgres> for Int {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 23,
            array_oid: 1007,
        }
    }
}

impl AsSql<Postgres> for i32 {
    type Type = Int;
}

impl ToSql<Postgres, Int> for i32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Postgres, Int> for i32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        BigEndian::read_i32(buf.unwrap())
    }
}

impl SqlType<Postgres> for BigInt {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 20,
            array_oid: 1016,
        }
    }
}

impl AsSql<Postgres> for i64 {
    type Type = BigInt;
}

impl ToSql<Postgres, BigInt> for i64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Postgres, BigInt> for i64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        BigEndian::read_i64(buf.unwrap())
    }
}

impl SqlType<Postgres> for Real {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 700,
            array_oid: 1021,
        }
    }
}

impl AsSql<Postgres> for f32 {
    type Type = Real;
}

impl ToSql<Postgres, Real> for f32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i32).to_sql(buf)
    }
}

impl FromSql<Postgres, BigInt> for f32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f32::from_bits(i32::from_sql(buf) as u32)
    }
}

impl SqlType<Postgres> for Double {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 701,
            array_oid: 1022,
        }
    }
}

impl AsSql<Postgres> for f64 {
    type Type = Double;
}

impl ToSql<Postgres, Double> for f64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i64).to_sql(buf)
    }
}

impl FromSql<Postgres, Double> for f64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f64::from_bits(i64::from_sql(buf) as u64)
    }
}
