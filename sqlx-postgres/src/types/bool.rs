use sqlx_core::{decode, encode, Decode, Encode, Type};

use crate::{PgOutput, PgRawValue, PgRawValueFormat, PgTypeId, PgTypeInfo, Postgres};

// https://www.postgresql.org/docs/current/datatype-boolean.html

impl Type<Postgres> for bool {
    fn type_id() -> PgTypeId
    where
        Self: Sized,
    {
        PgTypeId::BOOLEAN
    }
}

impl Encode<Postgres> for bool {
    fn encode(&self, _ty: &PgTypeInfo, out: &mut PgOutput<'_>) -> encode::Result {
        out.buffer().push(*self as u8);

        Ok(encode::IsNull::No)
    }
}

impl<'r> Decode<'r, Postgres> for bool {
    fn decode(value: PgRawValue<'r>) -> decode::Result<Self> {
        Ok(match value.format() {
            PgRawValueFormat::Binary => value.as_bytes()?[0] != 0,
            PgRawValueFormat::Text => match value.as_str()? {
                "t" => true,
                "f" => false,

                s => {
                    return Err(decode::Error::msg(format!(
                        "unexpected value {:?} for `boolean`",
                        s
                    )));
                }
            },
        })
    }
}
