use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

// ── NaiveDateTime ───────────────────────────────────────────────────────────

impl Type<Mssql> for NaiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIME2")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIME2" | "DATETIME" | "SMALLDATETIME")
    }
}

impl Encode<'_, Mssql> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveDateTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for NaiveDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveDateTime(v) => Ok(*v),
            MssqlData::DateTimeFixedOffset(v) => Ok(v.naive_utc()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetime, got {:?}", value.data).into()),
        }
    }
}

// ── NaiveDate ───────────────────────────────────────────────────────────────

impl Type<Mssql> for NaiveDate {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATE")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "DATE"
    }
}

impl Encode<'_, Mssql> for NaiveDate {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveDate(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for NaiveDate {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveDate(v) => Ok(*v),
            MssqlData::NaiveDateTime(v) => Ok(v.date()),
            MssqlData::DateTimeFixedOffset(v) => Ok(v.naive_utc().date()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected date, got {:?}", value.data).into()),
        }
    }
}

// ── NaiveTime ───────────────────────────────────────────────────────────────

impl Type<Mssql> for NaiveTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("TIME")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "TIME"
    }
}

impl Encode<'_, Mssql> for NaiveTime {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for NaiveTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveTime(v) => Ok(*v),
            MssqlData::NaiveDateTime(v) => Ok(v.time()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected time, got {:?}", value.data).into()),
        }
    }
}

// ── DateTime<Utc> ───────────────────────────────────────────────────────────

impl Type<Mssql> for DateTime<Utc> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIME2")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIME2" | "DATETIMEOFFSET")
    }
}

impl Encode<'_, Mssql> for DateTime<Utc> {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveDateTime(self.naive_utc()));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for DateTime<Utc> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveDateTime(v) => Ok(v.and_utc()),
            MssqlData::DateTimeFixedOffset(v) => Ok(v.with_timezone(&Utc)),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetime, got {:?}", value.data).into()),
        }
    }
}

// ── DateTime<FixedOffset> ───────────────────────────────────────────────────

impl Type<Mssql> for DateTime<FixedOffset> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIMEOFFSET")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIMEOFFSET" | "DATETIME2")
    }
}

impl Encode<'_, Mssql> for DateTime<FixedOffset> {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::DateTimeFixedOffset(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for DateTime<FixedOffset> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::DateTimeFixedOffset(v) => Ok(*v),
            MssqlData::NaiveDateTime(v) => {
                // Assume UTC if no offset information
                let utc = v.and_utc();
                Ok(utc.with_timezone(
                    &FixedOffset::east_opt(0).expect("UTC offset 0 is always valid"),
                ))
            }
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetimeoffset, got {:?}", value.data).into()),
        }
    }
}
