use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

impl Type<Mssql> for bool {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("BIT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.base_name(),
            "BIT" | "TINYINT" | "INT" | "SMALLINT" | "BIGINT"
        )
    }
}

impl Encode<'_, Mssql> for bool {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::Bool(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for bool {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::Bool(v) => Ok(*v),
            MssqlData::U8(v) => Ok(*v != 0),
            MssqlData::I16(v) => Ok(*v != 0),
            MssqlData::I32(v) => Ok(*v != 0),
            MssqlData::I64(v) => Ok(*v != 0),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected bool-compatible type, got {:?}", value.data).into()),
        }
    }
}
