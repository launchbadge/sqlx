use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;

struct TsVector(String);

impl TsVector {
    fn from_slice(raw: &[u8]) -> Result<Self, BoxDynError> {
        let result = std::str::from_utf8(raw)?;
        Ok(TsVector(result.to_owned()))
    }

    fn from_str(result: &str) -> Result<Self, BoxDynError> {
        Ok(TsVector(result.to_owned()))
    }
}

impl Type<Postgres> for TsVector {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::TS_VECTOR
    }
}

impl PgHasArrayType for TsVector {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::TS_VECTOR_ARRAY
    }
}

impl Encode<'_, Postgres> for TsVector {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for TsVector {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => TsVector::from_slice(value.as_bytes()?),
            PgValueFormat::Text => TsVector::from_str(value.as_str()?),
        }
        .map_err(Into::into)
    }
}
