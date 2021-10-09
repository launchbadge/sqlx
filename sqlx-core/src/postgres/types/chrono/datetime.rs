use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;
use chrono::{
    DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc,
};
use std::mem;

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

fn is_timestamptz_output(value_as_str: &str) -> bool {
    // This function returns true if the value contains a time-zone specifier
    // This is given for timestamptz for some reason
    return value_as_str.contains('+') || value_as_str.split('-').count() == 4usize;
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
                    if is_timestamptz_output(s) {
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
        Ok(<DateTime<Utc> as Decode<Postgres>>::decode(value)?.with_timezone(&Local))
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<Utc> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let epoch = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0);
                let us = Decode::<Postgres>::decode(value)?;
                epoch + Duration::microseconds(us)
            }

            PgValueFormat::Text => {
                let s = value.as_str()?;
                DateTime::parse_from_str(
                    s,
                    if is_timestamptz_output(s) {
                        // Note that because the output format differs from RFC3339,
                        // `DateTime::parse_from_rfc3339` is not available
                        "%Y-%m-%d %H:%M:%S%.f%#z"
                    } else {
                        "%Y-%m-%d %H:%M:%S%.f"
                    },
                )?
                .with_timezone(&Utc)
            }
        })
    }
}

impl<'r> Decode<'r, Postgres> for DateTime<FixedOffset> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let datetime_naive_utc = <DateTime<Utc> as Decode<Postgres>>::decode(value)?.naive_utc();
        Ok(Utc.fix().from_utc_datetime(&datetime_naive_utc))
    }
}

#[cfg(test)]
mod test {
    use super::is_timestamptz_output;
    #[test]
    fn distinguishing_timestamptz_output() {
        assert!(is_timestamptz_output("2020-01-01 02:01:01+01"));
        assert!(is_timestamptz_output("2020-01-01 02:01:01-01"));
        assert!(!is_timestamptz_output("2020-01-01 02:01:01"));
    }
}
