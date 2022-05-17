use byteorder::{ByteOrder, LittleEndian};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mysql::protocol::text::ColumnType;
use crate::mysql::{MySql, MySqlTypeInfo, MySqlValueFormat, MySqlValueRef};
use crate::types::Type;

fn real_compatible(ty: &MySqlTypeInfo) -> bool {
    matches!(
        ty.r#type,
        ColumnType::Float | ColumnType::Double | ColumnType::NewDecimal
    )
}

impl Type<MySql> for f32 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Float)
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        real_compatible(ty)
    }
}

impl Type<MySql> for f64 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::binary(ColumnType::Double)
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        real_compatible(ty)
    }
}

impl Encode<'_, MySql> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Encode<'_, MySql> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MySql> for f32 {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(
            if value.format() == MySqlValueFormat::Text
                || value.type_info.r#type == ColumnType::NewDecimal
            {
                value.as_str()?.parse()?
            } else {
                let buf = value.as_bytes()?;

                if buf.len() == 8 {
                    // MySQL can return 8-byte DOUBLE values for a FLOAT
                    // We take and truncate to f32 as that's the same behavior as *in* MySQL
                    LittleEndian::read_f64(buf) as f32
                } else {
                    LittleEndian::read_f32(buf)
                }
            },
        )
    }
}

impl Decode<'_, MySql> for f64 {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(
            if value.format() == MySqlValueFormat::Text
                || value.type_info.r#type == ColumnType::NewDecimal
            {
                value.as_str()?.parse()?
            } else {
                LittleEndian::read_f64(value.as_bytes()?)
            },
        )
    }
}
