use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::database::{HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

impl Type<Any> for i16 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::SmallInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl<'q> Encode<'q, Any> for i16 {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::SmallInt(*self));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for i16 {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for i32 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Integer,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl<'q> Encode<'q, Any> for i32 {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::Integer(*self));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for i32 {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}

impl Type<Any> for i64 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::BigInt,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        ty.kind().is_integer()
    }
}

impl<'q> Encode<'q, Any> for i64 {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::BigInt(*self));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for i64 {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        value.kind.try_integer()
    }
}
