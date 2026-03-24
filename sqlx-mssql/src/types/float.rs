use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

fn real_compatible(ty: &MssqlTypeInfo) -> bool {
    matches!(ty.base_name(), "REAL" | "FLOAT" | "MONEY" | "SMALLMONEY")
}

impl Type<Mssql> for f32 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("REAL")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        real_compatible(ty)
    }
}

impl Encode<'_, Mssql> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::F32(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for f32 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::F32(v) => Ok(*v),
            #[allow(clippy::cast_possible_truncation)]
            MssqlData::F64(v) => Ok(*v as f32),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected float, got {:?}", value.data).into()),
        }
    }
}

impl Type<Mssql> for f64 {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("FLOAT")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        real_compatible(ty)
    }
}

impl Encode<'_, Mssql> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::F64(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for f64 {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::F32(v) => Ok(f64::from(*v)),
            MssqlData::F64(v) => Ok(*v),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected float, got {:?}", value.data).into()),
        }
    }
}
