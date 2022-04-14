use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::types::time::PG_EPOCH;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;
use std::mem;
use time::macros::format_description;
use time::{Date, Duration};

impl Type<Postgres> for Date {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::DATE
    }
}

impl PgHasArrayType for Date {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::DATE_ARRAY
    }
}

impl Encode<'_, Postgres> for Date {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // DATE is encoded as the days since epoch
        let days = (*self - PG_EPOCH).whole_days() as i32;
        Encode::<Postgres>::encode(&days, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i32>()
    }
}

impl<'r> Decode<'r, Postgres> for Date {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // DATE is encoded as the days since epoch
                let days: i32 = Decode::<Postgres>::decode(value)?;
                PG_EPOCH + Duration::days(days.into())
            }

            PgValueFormat::Text => Date::parse(
                value.as_str()?,
                &format_description!("[year]-[month]-[day]"),
            )?,
        })
    }
}
