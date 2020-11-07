use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    mssql::{
        protocol::type_info::{DataType, TypeInfo},
        Mssql, MssqlTypeInfo, MssqlValueRef,
    },
    types::Type,
};
use bytes::Buf;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

impl Type<Mssql> for NaiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::DateTime, 8))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::DateTime | DataType::DateTimeN) && ty.0.size == 8
    }
}

impl<'r> Decode<'r, Mssql> for NaiveDateTime {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut buf = value.as_bytes()?;
        let days = buf.get_i32_le();
        let ticks = buf.get_u32_le();
        Ok(NaiveDateTime::new(
            from_days(days.into(), 1900),
            from_sec_fragments(ticks.into()),
        ))
    }
}

impl Encode<'_, Mssql> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let date = self.date();
        let time = self.time();
        let days = to_days(date, 1900) as i32;
        let seconds_fragments = to_sec_fragments(time);
        buf.extend_from_slice(&days.to_le_bytes());
        buf.extend_from_slice(&seconds_fragments.to_le_bytes());
        IsNull::No
    }

    fn size_hint(&self) -> usize {
        8
    }
}

#[inline]
fn from_days(days: i64, start_year: i32) -> NaiveDate {
    NaiveDate::from_ymd(start_year, 1, 1) + chrono::Duration::days(days as i64)
}

#[inline]
fn from_sec_fragments(sec_fragments: i64) -> NaiveTime {
    NaiveTime::from_hms(0, 0, 0) + chrono::Duration::nanoseconds(sec_fragments * (1e9 as i64) / 300)
}

#[inline]
fn to_days(date: NaiveDate, start_year: i32) -> i64 {
    date.signed_duration_since(NaiveDate::from_ymd(start_year, 1, 1))
        .num_days()
}

#[inline]
fn to_sec_fragments(time: NaiveTime) -> i64 {
    time.signed_duration_since(NaiveTime::from_hms(0, 0, 0))
        .num_nanoseconds()
        .unwrap()
        * 300
        / (1e9 as i64)
}
