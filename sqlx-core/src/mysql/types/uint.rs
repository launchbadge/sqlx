use byteorder::{ByteOrder, LittleEndian};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mysql::protocol::text::{ColumnFlags, ColumnType};
use crate::mysql::{MySql, MySqlTypeInfo, MySqlValueFormat, MySqlValueRef};
use crate::types::Type;

fn uint_type_info(ty: ColumnType) -> MySqlTypeInfo {
    MySqlTypeInfo {
        r#type: ty,
        flags: ColumnFlags::BINARY | ColumnFlags::UNSIGNED,
        char_set: 63,
    }
}

fn uint_accepts(ty: &MySqlTypeInfo) -> bool {
    matches!(
        ty.r#type,
        ColumnType::Tiny
            | ColumnType::Short
            | ColumnType::Long
            | ColumnType::Int24
            | ColumnType::LongLong
    ) && ty.flags.contains(ColumnFlags::UNSIGNED)
}

impl Type<MySql> for u8 {
    fn type_info() -> MySqlTypeInfo {
        uint_type_info(ColumnType::Tiny)
    }
}

impl Type<MySql> for u16 {
    fn type_info() -> MySqlTypeInfo {
        uint_type_info(ColumnType::Short)
    }
}

impl Type<MySql> for u32 {
    fn type_info() -> MySqlTypeInfo {
        uint_type_info(ColumnType::Long)
    }
}

impl Type<MySql> for u64 {
    fn type_info() -> MySqlTypeInfo {
        uint_type_info(ColumnType::LongLong)
    }
}

impl Encode<'_, MySql> for u8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for u16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for u32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for u64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MySql> for u8 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        uint_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => value.as_bytes()?[0] as u8,
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Decode<'_, MySql> for u16 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        uint_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => LittleEndian::read_u16(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Decode<'_, MySql> for u32 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        uint_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => LittleEndian::read_u32(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Decode<'_, MySql> for u64 {
    fn accepts(ty: &MySqlTypeInfo) -> bool {
        uint_accepts(ty)
    }

    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            MySqlValueFormat::Binary => LittleEndian::read_u64(value.as_bytes()?),
            MySqlValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
