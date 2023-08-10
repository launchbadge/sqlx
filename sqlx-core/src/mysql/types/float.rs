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
        let as_f64 = <f64 as Decode<'_, MySql>>::decode(value)?;
        Ok(as_f64 as f32)
    }
}

impl Decode<'_, MySql> for f64 {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match (value.format(), value.type_info.r#type) {
            (MySqlValueFormat::Binary, ColumnType::Float | ColumnType::Double) => {
                let buf = value.as_bytes()?;
                match buf.len() {
                    4 => f32::from_le_bytes(buf.try_into()?) as f64,
                    8 => f64::from_le_bytes(buf.try_into()?),
                    _ => {
                        return Err(
                            format!("float value buffer of unexpected size: {:02X?}", buf).into(),
                        )
                    }
                }
            }
            _ => {
                let str_val = value.as_str()?;
                let parsed = str_val.parse()?;
                parsed
            }
        })
    }
}
