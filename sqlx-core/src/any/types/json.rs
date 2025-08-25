use crate::any::{Any, AnyArgumentBuffer, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind, AnyValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::{Json, Type};
use serde::{Deserialize, Serialize};

impl<T> Type<Any> for Json<T> {
    fn type_info() -> AnyTypeInfo {
        AnyTypeInfo {
            kind: AnyTypeInfoKind::Text,
        }
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        matches!(ty.kind, AnyTypeInfoKind::Text | AnyTypeInfoKind::Blob)
    }
}

impl<T> Encode<'_, Any> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(&self, buf: &mut AnyArgumentBuffer<'_>) -> Result<IsNull, BoxDynError> {
        let json_string = self.encode_to_string()?;
        buf.0.push(AnyValueKind::Text(json_string.into()));
        Ok(IsNull::No)
    }
}

impl<'r, T> Decode<'_, Any> for Json<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn decode(value: AnyValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.kind {
            AnyValueKind::Text(text) => Json::decode_from_string(&text.into_owned()),
            AnyValueKind::Blob(blob) => Json::decode_from_bytes(&blob.into_owned()),
            other => other.unexpected(),
        }
    }
}
