use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

// ── Date ───────────────────────────────────────────────────────────────────

impl Type<Mssql> for Date {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATE")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "DATE"
    }
}

impl Encode<'_, Mssql> for Date {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimeDate(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for Date {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimeDate(v) => Ok(*v),
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.date()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected date, got {:?}", value.data).into()),
        }
    }
}

// ── Time ───────────────────────────────────────────────────────────────────

impl Type<Mssql> for Time {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("TIME")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "TIME"
    }
}

impl Encode<'_, Mssql> for Time {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimeTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for Time {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimeTime(v) => Ok(*v),
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.time()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected time, got {:?}", value.data).into()),
        }
    }
}

// ── PrimitiveDateTime ──────────────────────────────────────────────────────

impl Type<Mssql> for PrimitiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIME2")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIME2" | "DATETIME" | "SMALLDATETIME")
    }
}

impl Encode<'_, Mssql> for PrimitiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimePrimitiveDateTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for PrimitiveDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimePrimitiveDateTime(v) => Ok(*v),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetime, got {:?}", value.data).into()),
        }
    }
}

// ── OffsetDateTime ─────────────────────────────────────────────────────────

impl Type<Mssql> for OffsetDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIMEOFFSET")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIMEOFFSET" | "DATETIME2")
    }
}

impl Encode<'_, Mssql> for OffsetDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimeOffsetDateTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for OffsetDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimeOffsetDateTime(v) => Ok(*v),
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.assume_utc()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetimeoffset, got {:?}", value.data).into()),
        }
    }
}
