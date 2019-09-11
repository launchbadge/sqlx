use super::{MariaDb, MariaDbTypeMetadata};
use crate::{
    deserialize::FromSql,
    mariadb::protocol::{FieldType, ParameterFlag},
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use byteorder::{BigEndian, ByteOrder};

impl HasSqlType<i16> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_SHORT
            field_type: FieldType(2),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl ToSql<MariaDb> for i16 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<MariaDb> for i16 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i16(buf.unwrap())
    }
}

impl HasSqlType<i32> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(3),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl ToSql<MariaDb> for i32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<MariaDb> for i32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i32(buf.unwrap())
    }
}

impl HasSqlType<i64> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONGLONG
            field_type: FieldType(8),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl ToSql<MariaDb> for i64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_be_bytes());

        IsNull::No
    }
}

impl FromSql<MariaDb> for i64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        BigEndian::read_i64(buf.unwrap())
    }
}

impl HasSqlType<f32> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_FLOAT
            field_type: FieldType(4),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl ToSql<MariaDb> for f32 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i32).to_sql(buf)
    }
}

impl FromSql<MariaDb> for f32 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f32::from_bits(i32::from_sql(buf) as u32)
    }
}

impl HasSqlType<f64> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_DOUBLE
            field_type: FieldType(4),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl ToSql<MariaDb> for f64 {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        (self.to_bits() as i64).to_sql(buf)
    }
}

impl FromSql<MariaDb> for f64 {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        f64::from_bits(i64::from_sql(buf) as u64)
    }
}
