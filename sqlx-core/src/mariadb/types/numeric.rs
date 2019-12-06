use super::{MariaDb, MariaDbTypeMetadata};
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    mariadb::protocol::{FieldType, ParameterFlag},
    types::HasSqlType,
};
use byteorder::{ByteOrder, LittleEndian};

impl HasSqlType<i8> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            field_type: FieldType(1),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MariaDb> for i8 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self as u8);

        IsNull::No
    }
}

impl Decode<MariaDb> for i8 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        buf.unwrap()[0] as i8
    }
}

impl HasSqlType<u8> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            field_type: FieldType(1),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MariaDb> for u8 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self);

        IsNull::No
    }
}

impl Decode<MariaDb> for u8 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        buf.unwrap()[0]
    }
}

impl HasSqlType<i16> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(2),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MariaDb> for i16 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MariaDb> for i16 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_i16(buf.unwrap())
    }
}

impl HasSqlType<u16> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(2),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MariaDb> for u16 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MariaDb> for u16 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_u16(buf.unwrap())
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

impl Encode<MariaDb> for i32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MariaDb> for i32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_i32(buf.unwrap())
    }
}

impl HasSqlType<u32 > for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(3),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MariaDb> for u32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MariaDb> for u32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_u32(buf.unwrap())
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

impl Encode<MariaDb> for i64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MariaDb> for i64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_i64(buf.unwrap())
    }
}

impl HasSqlType<u64> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONGLONG
            field_type: FieldType(8),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MariaDb> for u64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MariaDb> for u64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_u64(buf.unwrap())
    }
}

impl HasSqlType<f32> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_FLOAT
            field_type: FieldType(4),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MariaDb> for f32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <i32 as Encode<MariaDb>>::encode(&(self.to_bits() as i32), buf)
    }
}

impl Decode<MariaDb> for f32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        f32::from_bits(<i32 as Decode<MariaDb>>::decode(buf) as u32)
    }
}

impl HasSqlType<f64> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_DOUBLE
            field_type: FieldType(4),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MariaDb> for f64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <i64 as Encode<MariaDb>>::encode(&(self.to_bits() as i64), buf)
    }
}

impl Decode<MariaDb> for f64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        f64::from_bits(<i64 as Decode<MariaDb>>::decode(buf) as u64)
    }
}
