use byteorder::{BigEndian, ByteOrder};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};

fn int_decode(value: PgValueRef<'_>) -> Result<i64, BoxDynError> {
    Ok(match value.format() {
        PgValueFormat::Text => value.as_str()?.parse()?,
        PgValueFormat::Binary => {
            let buf = value.as_bytes()?;
            BigEndian::read_int(buf, buf.len())
        }
    })
}

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
        if (value.format() == PgValueFormat::Text) && (value.as_bytes()?.is_empty()) {
            return Ok(Default::default());
        }

        int_decode(value)?.try_into().map_err(Into::into)
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
        int_decode(value)?.try_into().map_err(Into::into)
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
        int_decode(value)?.try_into().map_err(Into::into)
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
        int_decode(value)
    }
}
