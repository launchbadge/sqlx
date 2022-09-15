use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::database::{HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use std::borrow::Cow;

impl Type<Any> for [u8] {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Blob,
        }
    }
}

impl<'q> Encode<'q, Any> for &'q [u8] {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::Blob((*self).into()));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for &'r [u8] {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Blob(Cow::Borrowed(blob)) => Ok(blob),
            // This shouldn't happen in practice, it means the user got an `AnyValueRef`
            // constructed from an owned `Vec<u8>` which shouldn't be allowed by the API.
            AnyValueKind::Blob(Cow::Owned(_text)) => {
                panic!("attempting to return a borrow that outlives its buffer")
            }
            other => other.unexpected(),
        }
    }
}

impl Type<Any> for Vec<u8> {
    fn type_info() -> AnyTypeInfo {
        <[u8] as Type<Any>>::type_info()
    }
}

impl<'q> Encode<'q, Any> for Vec<u8> {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::Blob(Cow::Owned(self.clone())));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for Vec<u8> {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Blob(blob) => Ok(blob.into_owned()),
            other => other.unexpected(),
        }
    }
}
