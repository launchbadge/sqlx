use rust_decimal::Decimal;

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

impl Type<Mssql> for Decimal {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DECIMAL")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.base_name(),
            "DECIMAL" | "NUMERIC" | "MONEY" | "SMALLMONEY"
        )
    }
}

impl Encode<'_, Mssql> for Decimal {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::Decimal(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for Decimal {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::Decimal(v) => Ok(*v),
            MssqlData::I32(v) => Ok(Decimal::from(*v)),
            MssqlData::I64(v) => Ok(Decimal::from(*v)),
            MssqlData::F64(v) => Decimal::try_from(*v)
                .map_err(|e| format!("failed to convert f64 to Decimal: {e}").into()),
            MssqlData::String(ref s) => s
                .parse::<Decimal>()
                .map_err(|e| format!("failed to parse Decimal from string: {e}").into()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected DECIMAL, got {:?}", value.data).into()),
        }
    }
}
