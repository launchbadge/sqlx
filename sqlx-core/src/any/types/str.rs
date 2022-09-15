use crate::any::types::str;
use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::database::{HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use std::borrow::Cow;

impl Type<Any> for str {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Text,
        }
    }
}

impl<'a> Encode<'a, Any> for &'a str {
    fn encode(self, buf: &mut <Any as HasArguments<'a>>::ArgumentBuffer) -> IsNull
    where
        Self: Sized,
    {
        buf.0.push(AnyValueKind::Text(self.into()));
        IsNull::No
    }

    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'a>>::ArgumentBuffer) -> IsNull {
        (*self).encode(buf)
    }
}

impl<'a> Decode<'a, Any> for &'a str {
    fn decode(value: <Any as HasValueRef<'a>>::ValueRef) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Text(Cow::Borrowed(text)) => Ok(text),
            // This shouldn't happen in practice, it means the user got an `AnyValueRef`
            // constructed from an owned `String` which shouldn't be allowed by the API.
            AnyValueKind::Text(Cow::Owned(_text)) => {
                panic!("attempting to return a borrow that outlives its buffer")
            }
            other => other.unexpected(),
        }
    }
}

impl Type<Any> for String {
    fn type_info() -> AnyTypeInfo {
        <str as Type<Any>>::type_info()
    }
}

impl<'q> Encode<'q, Any> for String {
    fn encode_by_ref(&self, buf: &mut <Any as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.0.push(AnyValueKind::Text(Cow::Owned(self.clone())));
        IsNull::No
    }
}

impl<'r> Decode<'r, Any> for String {
    fn decode(value: <Any as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Text(text) => Ok(text.into_owned()),
            other => other.unexpected(),
        }
    }
}
