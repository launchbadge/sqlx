use crate::aurora::type_info::AuroraType;
use crate::aurora::{Aurora, AuroraTypeInfo, AuroraValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::types::Type;

use bytes::Bytes;
use rusoto_rds_data::{Field, SqlParameter};

impl Type<Aurora> for [u8] {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::Blob)
    }
}

impl Type<Aurora> for Vec<u8> {
    fn type_info() -> AuroraTypeInfo {
        <[u8] as Type<Aurora>>::type_info()
    }
}

impl Encode<'_, Aurora> for &'_ [u8] {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        buf.push(SqlParameter {
            value: Some(Field {
                blob_value: Some(Bytes::from(self.to_vec())),
                ..Default::default()
            }),
            ..Default::default()
        });

        IsNull::No
    }
}

impl Encode<'_, Aurora> for Vec<u8> {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        <&[u8] as Encode<Aurora>>::encode(self, buf)
    }
}

impl<'r> Decode<'r, Aurora> for &'r [u8] {
    fn decode(value: AuroraValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value
            .field
            .blob_value
            .as_ref()
            .ok_or_else(|| Error::Decode("Not a blob value".into()))?)
    }
}

impl Decode<'_, Aurora> for Vec<u8> {
    fn decode(value: AuroraValueRef<'_>) -> Result<Self, BoxDynError> {
        <&[u8] as Decode<Aurora>>::decode(value).map(|v| v.to_vec())
    }
}
