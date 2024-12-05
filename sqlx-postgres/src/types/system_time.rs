use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use std::time::{Duration, SystemTime};

impl Type<Postgres> for SystemTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ
    }
}

impl PgHasArrayType for SystemTime {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TIMESTAMPTZ_ARRAY
    }
}

impl Encode<'_, Postgres> for SystemTime {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // TIMESTAMP is encoded as the microseconds since the epoch
        let micros = self.duration_since(SystemTime::UNIX_EPOCH)?.as_micros();
        let micros = i64::try_from(micros)
            .map_err(|_| format!("SystemTime out of range for Postgres: {self:?}"))?;
        Encode::<Postgres>::encode(micros, buf)
    }

    fn size_hint(&self) -> usize {
        size_of::<i64>()
    }
}

impl<'r> Decode<'r, Postgres> for SystemTime {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                // TIMESTAMP is encoded as the microseconds since the epoch
                let us: i64 = Decode::<Postgres>::decode(value)?;
                let us = u64::try_from(us)
                    .map_err(|_| "Postgres TIMESTAMPTZ out of range for SystemTime (SystemTime only supports timestamps after UNIX epoch)")?;
                SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_micros(us))
                    .ok_or("Postgres TIMESTAMPTZ out of range for SystemTime")?
            }
            PgValueFormat::Text => {
                // std has no datetime parsing
                // We rely on chrono or time if they are available
                #[cfg(feature = "chrono")]
                {
                    chrono::DateTime::<chrono::Utc>::decode(value)?.into()
                }
                #[cfg(all(not(feature = "chrono"), feature = "time"))]
                {
                    time::OffsetDateTime::decode(value)?.into()
                }
                #[cfg(all(not(feature = "chrono"), not(feature = "time")))]
                return Err(
                    "not implemented: decode to SystemTime in text mode (unprepared queries)"
                        .into(),
                );
            }
        })
    }
}
