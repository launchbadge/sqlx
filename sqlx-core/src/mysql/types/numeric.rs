use super::{MySql, MariaDbTypeMetadata};
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    mysql::protocol::{FieldType, ParameterFlag},
    types::HasSqlType,
};
use byteorder::{ByteOrder, LittleEndian};

impl HasSqlType<i8> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            field_type: FieldType::MYSQL_TYPE_TINY,
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for i8 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self as u8);

        IsNull::No
    }
}

impl Decode<MySql> for i8 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        buf.unwrap()[0] as i8
    }
}

impl HasSqlType<u8> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            field_type: FieldType(1),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MySql> for u8 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self);

        IsNull::No
    }
}

impl Decode<MySql> for u8 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        buf.unwrap()[0]
    }
}

impl HasSqlType<i16> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(2),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for i16 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MySql> for i16 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_i16(buf.unwrap())
    }
}

impl HasSqlType<u16> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(2),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MySql> for u16 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MySql> for u16 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_u16(buf.unwrap())
    }
}

impl HasSqlType<i32> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(3),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for i32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MySql> for i32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_i32(buf.unwrap())
    }
}

impl HasSqlType<u32 > for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONG
            field_type: FieldType(3),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MySql> for u32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MySql> for u32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_u32(buf.unwrap())
    }
}

impl HasSqlType<i64> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONGLONG
            field_type: FieldType(8),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for i64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MySql> for i64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_i64(buf.unwrap())
    }
}

impl HasSqlType<u64> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_LONGLONG
            field_type: FieldType(8),
            param_flag: ParameterFlag::UNSIGNED,
        }
    }
}

impl Encode<MySql> for u64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<MySql> for u64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        LittleEndian::read_u64(buf.unwrap())
    }
}

impl HasSqlType<f32> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_FLOAT
            field_type: FieldType(4),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for f32 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <i32 as Encode<MySql>>::encode(&(self.to_bits() as i32), buf)
    }
}

impl Decode<MySql> for f32 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        f32::from_bits(<i32 as Decode<MySql>>::decode(buf) as u32)
    }
}

impl HasSqlType<f64> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_DOUBLE
            field_type: FieldType(4),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for f64 {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <i64 as Encode<MySql>>::encode(&(self.to_bits() as i64), buf)
    }
}

impl Decode<MySql> for f64 {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        f64::from_bits(<i64 as Decode<MySql>>::decode(buf) as u64)
    }
}
