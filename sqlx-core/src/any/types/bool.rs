use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::database::{HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

impl Type<Any> for bool {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Bool,
        }
    }
}

impl<'q> Encode<'q, Any> for bool {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::Bool(*self));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for bool {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Bool(b) => Ok(b),
            other => other.unexpected(),
        }
    }
}
