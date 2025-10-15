use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::database::Database;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

impl Type<Any> for u8 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::UnsignedTinyInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for u8 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::UnsignedTinyInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for u8 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for u16 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::UnsignedTinyInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for u16 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::UnsignedSmallInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for u16 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for u32 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::UnsignedInteger,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for u32 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::UnsignedInteger(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for u32 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for u64 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::UnsignedBigInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl Encode<'_, Any> for u64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::UnsignedBigInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for u64 {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}
