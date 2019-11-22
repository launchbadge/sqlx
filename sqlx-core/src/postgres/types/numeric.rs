use super::{Postgres, PostgresTypeFormat, PostgresTypeMetadata};
use crate::{
    decode::Decode,
    encode::{IsNull, Encode},
    types::HasSqlType,
};
use byteorder::{BigEndian, ByteOrder};

impl HasSqlType<i16> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 21,
            array_oid: 1005,
        }
    }
}

impl Encode<Postgres> for i16 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<Postgres> for i16 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i16(buf.unwrap())
    }
}

impl HasSqlType<i32> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 23,
            array_oid: 1007,
        }
    }
}

impl Encode<Postgres> for i32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<Postgres> for i32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i32(buf.unwrap())
    }
}

impl HasSqlType<i64> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 20,
            array_oid: 1016,
        }
    }
}

impl Encode<Postgres> for i64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<Postgres> for i64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i64(buf.unwrap())
    }
}

impl HasSqlType<f32> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 700,
            array_oid: 1021,
        }
    }
}

impl Encode<Postgres> for f32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <i32 as Encode<Postgres>>::to_sql(&(self.to_bits() as i32), buf)
    }
}

impl Decode<Postgres> for f32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        f32::from_bits(<i32 as Decode<Postgres>>::from_sql(buf) as u32)
    }
}

impl HasSqlType<f64> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 701,
            array_oid: 1022,
        }
    }
}

impl Encode<Postgres> for f64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <i64 as Encode<Postgres>>::to_sql(&(self.to_bits() as i64), buf)
    }
}

impl Decode<Postgres> for f64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        f64::from_bits(<i64 as Decode<Postgres>>::from_sql(buf) as u64)
    }
}
