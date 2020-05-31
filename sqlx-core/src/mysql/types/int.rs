use byteorder::{ByteOrder, LittleEndian};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mysql::protocol::text::{ColumnFlags, ColumnType};
use crate::mysql::{MySql, MySqlTypeInfo, MySqlValueFormat, MySqlValueRef};
use crate::types::Type;

fn int_accepts(ty: &MySqlTypeInfo) -> bool {
    matches!(
        ty.r#type,
        ColumnType::Tiny
            | ColumnType::Short
            | ColumnType::Long
            | ColumnType::Int24
            | ColumnType::LongLong
    ) && !ty.flags.contains(ColumnFlags::UNSIGNED)
}

impl Type<MySql> for i8 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Tiny)
    }
}

impl Type<MySql> for i16 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Short)
    }
}

impl Type<MySql> for i32 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Long)
    }
}

impl Type<MySql> for i64 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::LongLong)
    }
}

impl Encode<'_, MySql> for i8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for i16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for i32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for i64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MySql> for i8 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        int_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => value.as_bytes()?[0] as i8,
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Decode<'_, MySql> for i16 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        int_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => LittleEndian::read_i16(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Decode<'_, MySql> for i32 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        int_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => LittleEndian::read_i32(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Decode<'_, MySql> for i64 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        int_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => LittleEndian::read_i64(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
