use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

fn int_compatible(ty: &MssqlTypeInfo) -> bool {
    matches!(ty.base_name(), "TINYINT" | "SMALLINT" | "INT" | "BIGINT")
}

// u8 - MSSQL's TINYINT is unsigned (0-255)
impl Type<Mssql> for u8 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("TINYINT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Encode<'_, Mssql> for u8 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::U8(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for u8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::U8(v) => Ok(*v),
            MssqlData::I16(v) => Ok((*v).try_into()?),
            MssqlData::I32(v) => Ok((*v).try_into()?),
            MssqlData::I64(v) => Ok((*v).try_into()?),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected integer, got {:?}", value.data).into()),
        }
    }
}

// i8 - maps to TINYINT but only 0-127 range
impl Type<Mssql> for i8 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("TINYINT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Encode<'_, Mssql> for i8 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        if *self < 0 {
            return Err("MSSQL TINYINT is unsigned; cannot encode negative i8".into());
        }
        #[allow(clippy::cast_sign_loss)]
        buf.push(MssqlArgumentValue::U8(*self as u8));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for i8 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::U8(v) => Ok((*v).try_into()?),
            MssqlData::I16(v) => Ok((*v).try_into()?),
            MssqlData::I32(v) => Ok((*v).try_into()?),
            MssqlData::I64(v) => Ok((*v).try_into()?),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected integer, got {:?}", value.data).into()),
        }
    }
}

// i16
impl Type<Mssql> for i16 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("SMALLINT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Encode<'_, Mssql> for i16 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::I16(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for i16 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::U8(v) => Ok(i16::from(*v)),
            MssqlData::I16(v) => Ok(*v),
            MssqlData::I32(v) => Ok((*v).try_into()?),
            MssqlData::I64(v) => Ok((*v).try_into()?),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected integer, got {:?}", value.data).into()),
        }
    }
}

// i32
impl Type<Mssql> for i32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("INT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Encode<'_, Mssql> for i32 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::I32(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for i32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::U8(v) => Ok(i32::from(*v)),
            MssqlData::I16(v) => Ok(i32::from(*v)),
            MssqlData::I32(v) => Ok(*v),
            MssqlData::I64(v) => Ok((*v).try_into()?),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected integer, got {:?}", value.data).into()),
        }
    }
}

// i64
impl Type<Mssql> for i64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("BIGINT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Encode<'_, Mssql> for i64 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::I64(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for i64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::U8(v) => Ok(i64::from(*v)),
            MssqlData::I16(v) => Ok(i64::from(*v)),
            MssqlData::I32(v) => Ok(i64::from(*v)),
            MssqlData::I64(v) => Ok(*v),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected integer, got {:?}", value.data).into()),
        }
    }
}
