use byteorder::{BigEndian, ByteOrder};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};

impl Type<Postgres> for i8 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::CHAR
    }
}

impl PgHasArrayType for i8 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::CHAR_ARRAY
    }
}

impl Encode<'_, Postgres> for i8 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i8 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        // note: in the TEXT encoding, a value of "0" here is encoded as an empty string
        Ok(value.as_bytes()?.get(0).copied().unwrap_or_default() as i8)
    }
}

impl Type<Postgres> for i16 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT2
    }
}

impl PgHasArrayType for i16 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INT2_ARRAY
    }
}

impl Encode<'_, Postgres> for i16 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i16 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i16(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for i32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT4
    }
}

impl PgHasArrayType for i32 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INT4_ARRAY
    }
}

impl Encode<'_, Postgres> for i32 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for i64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INT8
    }
}

impl PgHasArrayType for i64 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::INT8_ARRAY
    }
}

impl Encode<'_, Postgres> for i64 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for i64 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_i64(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
