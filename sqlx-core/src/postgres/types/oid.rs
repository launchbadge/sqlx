use byteorder::{BigEndian, ByteOrder};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};
use crate::types::Type;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Oid(u32);

impl Oid {
    #[inline(always)]
    pub const fn new(oid: u32) -> Self {
        Self(oid)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub(crate) fn incr_one(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

impl Type<Postgres> for Oid {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::OID
    }
}

impl PgHasArrayType for Oid {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::OID_ARRAY
    }
}

impl Encode<'_, Postgres> for Oid {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(&self.0.to_be_bytes());

        IsNull::No
    }
}

impl Decode<'_, Postgres> for Oid {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(Self(match value.format() {
            PgValueFormat::Binary => BigEndian::read_u32(value.as_bytes()?),
            PgValueFormat::Text => value.as_str()?.parse()?,
        }))
    }
}
