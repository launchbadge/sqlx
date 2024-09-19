use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp, Zoned};
use std::mem;
use std::str::FromStr;

impl Type<Postgres> for Timestamp {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP
    }
}

impl Type<Postgres> for Zoned {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl PgHasArrayType for Timestamp {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP_ARRAY
    }
}

impl PgHasArrayType for Zoned {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ_ARRAY
    }
}

impl Encode<'_, Postgres> for Timestamp {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // TIMESTAMP is encoded as the microseconds since the epoch
        let micros = (*self - postgres_epoch_datetime()).get_microseconds();
        Encode::<Postgres>::encode(micros, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for Timestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let us = Decode::<Postgres>::decode(value)?;
                postgres_epoch_datetime() + SignedDuration::from_micros(us)
            }
            PgValueFormat::Text => {
                let s = value.as_str()?;
                parse_timestamp_text(s)?.timestamp()
            }
        })
    }
}

impl Encode<'_, Postgres> for Zoned {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        Encode::<Postgres>::encode(self.timestamp(), buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for Zoned {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                let naive = <Timestamp as Decode<Postgres>>::decode(value)?;
                naive.to_zoned(TimeZone::UTC)
            }
            PgValueFormat::Text => {
                let s = value.as_str()?;
                parse_timestamp_text(s)?
            }
        })
    }
}

#[inline]
fn parse_timestamp_text(input: &str) -> Result<Zoned, BoxDynError> {
    Ok(Zoned::strptime(
        if input.contains('+') || input.contains('-') {
            // Contains a time-zone specifier
            // This is given for timestamptz for some reason
            // Postgres already guarantees this to always be UTC
            "%Y-%m-%d %H:%M:%S%.f%#z"
        } else {
            "%Y-%m-%d %H:%M:%S%.f"
        },
        input,
    )?)
}

#[inline]
fn postgres_epoch_datetime() -> Timestamp {
    Timestamp::from_str("2000-01-01T00:00:00+00:00")
        .expect("expected 2000-01-01T00:00:00+00:00 to be a valid Timestamp")
}

#[test]
fn test_postgres_epoch_datetime() {
    let epoch_datetime = jiff::civil::datetime(2000, 1, 1, 0, 0, 0, 0)
        .intz("UTC")
        .unwrap();
    assert_eq!(postgres_epoch_datetime(), epoch_datetime.timestamp());
}
//
// #[test]
// fn test_parse_timestamp_text() {
//     let zoned = Zoned::from_str("2004-10-19 10:23:54+02").unwrap();
//     assert_eq!(
//         parse_timestamp_text("2024-09-19 03:58:43.152233+0000").unwrap(),
//         zoned
//     );
//
//     let zoned = Zoned::strptime("%Y-%m-%d %H:%M:%S%.f", "2021-01-01 00:00:00").unwrap();
//     assert_eq!(parse_timestamp_text("2021-01-01 00:00:00").unwrap(), zoned);
// }
