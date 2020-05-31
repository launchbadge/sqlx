use std::mem;

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

impl Type<Postgres> for NaiveTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIME
    }
}

impl Type<Postgres> for NaiveDate {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::DATE
    }
}

impl Type<Postgres> for NaiveDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP
    }
}

impl<Tz: TimeZone> Type<Postgres> for DateTime<Tz> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl Type<Postgres> for [NaiveTime] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIME_ARRAY
    }
}

impl Type<Postgres> for [NaiveDate] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::DATE_ARRAY
    }
}

impl Type<Postgres> for [NaiveDateTime] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP_ARRAY
    }
}

impl<Tz: TimeZone> Type<Postgres> for [DateTime<Tz>] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ_ARRAY
    }
}

impl Type<Postgres> for Vec<NaiveTime> {
    fn type_info() -> PgTypeInfo {
        <[NaiveTime] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for Vec<NaiveDate> {
    fn type_info() -> PgTypeInfo {
        <[NaiveDate] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for Vec<NaiveDateTime> {
    fn type_info() -> PgTypeInfo {
        <[NaiveDateTime] as Type<Postgres>>::type_info()
    }
}

impl<Tz: TimeZone> Type<Postgres> for Vec<DateTime<Tz>> {
    fn type_info() -> PgTypeInfo {
        <[DateTime<Tz>] as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for NaiveTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // TIME is encoded as the microseconds since midnight
        // NOTE: panic! is on overflow and 1 day does not have enough micros to overflow
        let us = (*self - NaiveTime::from_hms(0, 0, 0))
            .num_microseconds()
            .unwrap();
        Encode::<Postgres>::encode(&us, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<u64>()
    }
}

impl<'r> Decode<'r, Postgres> for NaiveTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIME is encoded as the microseconds since midnight
                let us: i64 = Decode::<Postgres>::decode(value)?;
                NaiveTime::from_hms(0, 0, 0) + Duration::microseconds(us)
            }

            PgValueFormat::Text => NaiveTime::parse_from_str(value.as_str()?, "%H:%M:%S%.f")?,
        })
    }
}

impl Encode<'_, Postgres> for NaiveDate {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // DATE is encoded as the days since epoch
        let days = (*self - NaiveDate::from_ymd(2000, 1, 1)).num_days() as i32;
        Encode::<Postgres>::encode(&days, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i32>()
    }
}

impl<'r> Decode<'r, Postgres> for NaiveDate {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // DATE is encoded as the days since epoch
                let days: i32 = Decode::<Postgres>::decode(value)?;
                NaiveDate::from_ymd(2000, 1, 1) + Duration::days(days.into())
            }

            PgValueFormat::Text => NaiveDate::parse_from_str(value.as_str()?, "%Y-%m-%d")?,
        })
    }
}

impl Encode<'_, Postgres> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // FIXME: We should *really* be returning an error, Encode needs to be fallible
        // TIMESTAMP is encoded as the microseconds since the epoch
        let epoch = NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
        let us = (*self - epoch)
            .num_microseconds()
            .unwrap_or_else(|| panic!("NaiveDateTime out of range for Postgres: {:?}", self));
        Encode::<Postgres>::encode(&us, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for NaiveDateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let epoch = NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
                let us = Decode::<Postgres>::decode(value)?;
                epoch + Duration::microseconds(us)
            }

            PgValueFormat::Text => {
                let s = value.as_str()?;
                NaiveDateTime::parse_from_str(
                    s,
                    if s.contains('+') {
                        // Contains a time-zone specifier
                        // This is given for timestamptz for some reason
                        // Postgres already guarantees this to always be UTC
                        "%Y-%m-%d %H:%M:%S%.f%#z"
                    } else {
                        "%Y-%m-%d %H:%M:%S%.f"
                    },
                )?
            }
        })
    }
}

impl<Tz: TimeZone> Encode<'_, Postgres> for DateTime<Tz> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        Encode::<Postgres>::encode(self.naive_utc(), buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<Local> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let naive = <NaiveDateTime as Decode<Postgres>>::decode(value)?;
        Ok(Local.from_utc_datetime(&naive))
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<Utc> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let naive = <NaiveDateTime as Decode<Postgres>>::decode(value)?;
        Ok(Utc.from_utc_datetime(&naive))
    }
}
