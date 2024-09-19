use std::mem;
use jiff::civil::Time;
use jiff::SignedDuration;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};

impl Type<Postgres> for Time {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIME
    }
}

impl PgHasArrayType for Time {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIME_ARRAY
    }
}

impl Encode<'_, Postgres> for Time {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // TIME is encoded as the microseconds since midnight
        let micros = (*self - Time::midnight()).get_microseconds();
        Encode::<Postgres>::encode(micros, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<u64>()
    }
}

impl<'r> Decode<'r, Postgres> for Time {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIME is encoded as the microseconds since midnight
                let us: i64 = Decode::<Postgres>::decode(value)?;
                Time::midnight() + SignedDuration::from_micros(us)
            }
            PgValueFormat::Text => Time::strptime("%H:%M:%S%.f", value.as_str()?)?,
        })
    }
}
