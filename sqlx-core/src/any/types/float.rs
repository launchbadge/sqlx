use crate::any::{Any, AnyArgumentBuffer, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind, AnyValueRef};
use crate::database::{HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

impl Type<Any> for f32 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Real,
        }
    }
}

impl<'q> Encode<'q, Any> for f32 {
    fn encode_by_ref(&self, buf: &mut AnyArgumentBuffer<'q>) -> IsNull {
        buf.0.push(AnyValueKind::Real(*self));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for f32 {
    fn decode(value: AnyValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Real(r) => Ok(r),
            other => other.unexpected(),
        }
    }
}

impl Type<Any> for f64 {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Double,
        }
    }
}

impl<'q> Encode<'q, Any> for f64 {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::Double(*self));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for f64 {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        match value.kind {
            // Widening is safe
            AnyValueKind::Real(r) => Ok(r as f64),
            AnyValueKind::Double(d) => Ok(d),
            other => other.unexpected(),
        }
    }
}
