use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind};
use crate::database::Database;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use std::sync::Arc;

impl Type<Any> for [u8] {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Blob,
        }
    }
}

impl<'q> Encode<'q, Any> for &'q [u8] {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::Blob(Arc::new(self.to_vec())));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for &'r [u8] {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Blob(blob) => Ok(blob.as_slice()),
            other => other.unexpected(),
        }
    }
}

impl Type<Any> for Vec<u8> {
    fn type_info() -> AnyTypeInfo {
        <[u8] as Type<Any>>::type_info()
    }
}

impl Encode<'_, Any> for Vec<u8> {
    fn encode_by_ref(
        &self,
        buf: &mut <Any as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        buf.0.push(AnyValueKind::Blob(Arc::new(self.clone())));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Any> for Vec<u8> {
    fn decode(value: <Any as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Blob(blob) => Ok(blob.as_ref().clone()),
            other => other.unexpected(),
        }
    }
}
