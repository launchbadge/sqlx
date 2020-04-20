use crate::decode::{Decode, Error};
use crate::postgres::{PgRawValue, Postgres};

impl<'r> Decode<'r, Postgres> for &'r str {
    #[inline]
    fn decode(value: PgRawValue<'r>) -> Result<Self, Error> {
        Ok(value.as_str()?)
    }
}

impl Decode<'_, Postgres> for String {
    #[inline]
    fn decode(value: PgRawValue<'_>) -> Result<Self, Error> {
        Ok(value.as_str()?.to_owned())
    }
}
