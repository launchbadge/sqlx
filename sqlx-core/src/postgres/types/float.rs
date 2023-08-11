use byteorder::{BigEndian, ByteOrder};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::types::numeric::PgNumeric;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;

impl Type<Postgres> for f32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::FLOAT4
    }
}

impl PgHasArrayType for f32 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::FLOAT4_ARRAY
    }
}

impl Encode<'_, Postgres> for f32 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for f32 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => BigEndian::read_f32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}

impl Type<Postgres> for f64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::FLOAT8
    }
    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::FLOAT4 || *ty == PgTypeInfo::FLOAT8 || *ty == PgTypeInfo::NUMERIC
    }
}

impl PgHasArrayType for f64 {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::FLOAT8_ARRAY
    }
}

impl Encode<'_, Postgres> for f64 {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for f64 {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(match value.format() {
            PgValueFormat::Binary => {
                if value.type_info == PgTypeInfo::NUMERIC {
                    return Ok(PgNumeric::decode(value.as_bytes()?)?.try_into()?);
                }
                let buf = value.as_bytes()?;
                match buf.len() {
                    8 => f64::from_be_bytes(buf.try_into()?),
                    4 => f64::from(f32::from_be_bytes(buf.try_into()?)),
                    _ => return Err("invalid buffer size".into()),
                }
            }
            PgValueFormat::Text => value.as_str()?.parse()?,
        })
    }
}
