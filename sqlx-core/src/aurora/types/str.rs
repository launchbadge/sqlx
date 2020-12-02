use crate::aurora::type_info::AuroraType;
use crate::aurora::{Aurora, AuroraTypeInfo, AuroraValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::types::Type;

use rusoto_rds_data::{Field, SqlParameter};

impl Type<Aurora> for str {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::String)
    }
}

impl Type<Aurora> for [&'_ str] {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::StringArray)
    }
}

impl Type<Aurora> for Vec<&'_ str> {
    fn type_info() -> AuroraTypeInfo {
        <[&str] as Type<Aurora>>::type_info()
    }

    fn compatible(ty: &AuroraTypeInfo) -> bool {
        <[&str] as Type<Aurora>>::compatible(ty)
    }
}

impl Encode<'_, Aurora> for &'_ str {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        buf.push(SqlParameter {
            value: Some(Field {
                string_value: Some(self.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        });

        IsNull::No
    }
}

impl<'r> Decode<'r, Aurora> for &'r str {
    fn decode(value: AuroraValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value
            .field
            .string_value
            .as_deref()
            .ok_or_else(|| Error::Decode("Not a str value".into()))?)
    }
}

impl Type<Aurora> for String {
    fn type_info() -> AuroraTypeInfo {
        <&str as Type<Aurora>>::type_info()
    }

    fn compatible(ty: &AuroraTypeInfo) -> bool {
        <&str as Type<Aurora>>::compatible(ty)
    }
}

impl Type<Aurora> for [String] {
    fn type_info() -> AuroraTypeInfo {
        <[&str] as Type<Aurora>>::type_info()
    }

    fn compatible(ty: &AuroraTypeInfo) -> bool {
        <[&str] as Type<Aurora>>::compatible(ty)
    }
}

impl Type<Aurora> for Vec<String> {
    fn type_info() -> AuroraTypeInfo {
        <[String] as Type<Aurora>>::type_info()
    }

    fn compatible(ty: &AuroraTypeInfo) -> bool {
        <[String] as Type<Aurora>>::compatible(ty)
    }
}

impl Encode<'_, Aurora> for String {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        <&str as Encode<Aurora>>::encode(&**self, buf)
    }
}

impl Decode<'_, Aurora> for String {
    fn decode(value: AuroraValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value
            .field
            .string_value
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::Decode("Not a str value".into()))?)
    }
}
