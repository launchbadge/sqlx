use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use jiff::civil::DateTime;
use jiff::tz::{Offset, TimeZone};
use jiff::{SignedDuration, Timestamp, Zoned};
use std::mem;
use std::str::FromStr;

impl Type<Postgres> for DateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP
    }
}

impl Type<Postgres> for Zoned {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl PgHasArrayType for DateTime {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMP_ARRAY
    }
}

impl PgHasArrayType for Zoned {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ_ARRAY
    }
}

impl Encode<'_, Postgres> for DateTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // TIMESTAMP is encoded as the microseconds since the epoch
        let micros = (*self - postgres_epoch_datetime()).get_microseconds();
        Encode::<Postgres>::encode(micros, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error parsing datetime {squashed:?}")]
struct ParseError {
    squashed: Vec<jiff::Error>,
}

impl<'r> Decode<'r, Postgres> for DateTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let us = Decode::<Postgres>::decode(value)?;
                Ok(postgres_epoch_datetime() + SignedDuration::from_micros(us))
            }
            PgValueFormat::Text => {
                let input = value.as_str()?;
                let mut squashed = vec![];
                match DateTime::strptime("%Y-%m-%d %H:%M:%S%.f", input) {
                    Ok(datetime) => return Ok(datetime),
                    Err(err) => squashed.push(err),
                }
                match DateTime::strptime("%Y-%m-%d %H:%M:%S%.f%#z", input) {
                    Ok(datetime) => return Ok(datetime),
                    Err(err) => squashed.push(err),
                }
                Err(Box::new(ParseError { squashed }))
            }
        }
    }
}

impl Encode<'_, Postgres> for Timestamp {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let datetime = Offset::UTC.to_datetime(*self);
        Encode::<Postgres>::encode(datetime, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for Timestamp {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                let naive = <DateTime as Decode<Postgres>>::decode(value)?;
                naive.to_zoned(TimeZone::UTC)?.timestamp()
            }
            PgValueFormat::Text => Timestamp::from_str(value.as_str()?)?,
        })
    }
}

const fn postgres_epoch_datetime() -> DateTime {
    DateTime::constant(2000, 1, 1, 0, 0, 0, 0)
}
