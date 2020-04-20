use byteorder::{BigEndian, ByteOrder};

use crate::database::Database;
use crate::decode::{Decode, Error};
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgRawBuffer, PgRawValue, PgTypeInfo, PgValueFormat, Postgres};

impl Encode<Postgres> for i16 {
    fn produces() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::INT2
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i16 {
    fn decode(value: PgRawValue<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i16(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Encode<Postgres> for u32 {
    fn produces() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::OID
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for u32 {
    fn decode(value: PgRawValue<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_u32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Encode<Postgres> for i32 {
    fn produces() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::INT4
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i32 {
    fn decode(value: PgRawValue<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Encode<Postgres> for i64 {
    fn produces() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::INT8
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i64 {
    fn decode(value: PgRawValue<'_>) -> Result<Self, Error> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i64(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
