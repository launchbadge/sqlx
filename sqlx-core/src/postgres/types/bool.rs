use crate::database::Database;
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgRawBuffer, PgRawValue, PgTypeInfo, PgValueFormat, Postgres};

impl Encode<Postgres> for bool {
    fn produces() -> PgTypeInfo {
        PgTypeInfo::BOOL
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        buf.push(*self as u8);
        IsNull::No
    }
}

impl Decode<'_, Postgres> for bool {
    fn decode(value: PgRawValue<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => value.as_bytes()?[0] != 0,

            PgValueFormat::Text => match value.as_str()? {
                "t" => true,
                "f" => false,

                s => {
                    return Err(format!("unexpected value {:?} for boolean", s).into());
                }
            },
        })
    }
}
