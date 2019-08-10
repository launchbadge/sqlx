use super::{Pg, PgTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::{AsSqlType, BigInt, Double, HasSqlType, Int, Real, SmallInt},
};
use byteorder::{BigEndian, ByteOrder};

impl HasSqlType<SmallInt> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 21,
            array_oid: 1005,
        }
    }
}

impl AsSqlType<Pg> for i16 {
    type SqlType = SmallInt;
}

impl ToSql<SmallInt, Pg> for i16 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<SmallInt, Pg> for i16 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        BigEndian::read_i16(buf.unwrap())
    }
}

impl HasSqlType<Int> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 23,
            array_oid: 1007,
        }
    }
}

impl AsSqlType<Pg> for i32 {
    type SqlType = Int;
}

impl ToSql<Int, Pg> for i32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Int, Pg> for i32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        BigEndian::read_i32(buf.unwrap())
    }
}

impl HasSqlType<BigInt> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 20,
            array_oid: 1016,
        }
    }
}

impl AsSqlType<Pg> for i64 {
    type SqlType = BigInt;
}

impl ToSql<BigInt, Pg> for i64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<BigInt, Pg> for i64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        BigEndian::read_i64(buf.unwrap())
    }
}

impl HasSqlType<Real> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 700,
            array_oid: 1021,
        }
    }
}

impl AsSqlType<Pg> for f32 {
    type SqlType = Real;
}

impl ToSql<Real, Pg> for f32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i32).to_sql(buf)
    }
}

impl FromSql<BigInt, Pg> for f32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f32::from_bits(i32::from_sql(buf) as u32)
    }
}

impl HasSqlType<Double> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 701,
            array_oid: 1022,
        }
    }
}

impl AsSqlType<Pg> for f64 {
    type SqlType = Double;
}

impl ToSql<Double, Pg> for f64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i64).to_sql(buf)
    }
}

impl FromSql<Double, Pg> for f64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f64::from_bits(i64::from_sql(buf) as u64)
    }
}
