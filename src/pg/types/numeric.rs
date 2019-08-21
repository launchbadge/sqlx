use super::{Pg, PgTypeMetadata, PgTypeFormat};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use byteorder::{BigEndian, ByteOrder};

impl HasSqlType<i16> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            format: PgTypeFormat::Binary,
            oid: 21,
            array_oid: 1005,
        }
    }
}

impl ToSql<Pg> for i16 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Pg> for i16 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i16(buf.unwrap())
    }
}

impl HasSqlType<i32> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            format: PgTypeFormat::Binary,
            oid: 23,
            array_oid: 1007,
        }
    }
}

impl ToSql<Pg> for i32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Pg> for i32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i32(buf.unwrap())
    }
}

impl HasSqlType<i64> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            format: PgTypeFormat::Binary,
            oid: 20,
            array_oid: 1016,
        }
    }
}

impl ToSql<Pg> for i64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<Pg> for i64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i64(buf.unwrap())
    }
}

impl HasSqlType<f32> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            format: PgTypeFormat::Binary,
            oid: 700,
            array_oid: 1021,
        }
    }
}

impl ToSql<Pg> for f32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i32).to_sql(buf)
    }
}

impl FromSql<Pg> for f32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f32::from_bits(i32::from_sql(buf) as u32)
    }
}

impl HasSqlType<f64> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            format: PgTypeFormat::Binary,
            oid: 701,
            array_oid: 1022,
        }
    }
}

impl ToSql<Pg> for f64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i64).to_sql(buf)
    }
}

impl FromSql<Pg> for f64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f64::from_bits(i64::from_sql(buf) as u64)
    }
}
