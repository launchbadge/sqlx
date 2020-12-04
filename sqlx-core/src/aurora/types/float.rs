use crate::aurora::type_info::AuroraType;
use crate::aurora::{Aurora, AuroraTypeInfo, AuroraValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::types::Type;

use rusoto_rds_data::{Field, SqlParameter};

impl Type<Aurora> for f64 {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::Double)
    }
}

impl Type<Aurora> for [f64] {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::DoubleArray)
    }
}

impl Type<Aurora> for Vec<f64> {
    fn type_info() -> AuroraTypeInfo {
        <[f64] as Type<Aurora>>::type_info()
    }
}

impl Encode<'_, Aurora> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        buf.push(SqlParameter {
            value: Some(Field {
                double_value: Some(*self),
                ..Default::default()
            }),
            ..Default::default()
        });

        IsNull::No
    }
}

impl Decode<'_, Aurora> for f64 {
    fn decode(value: AuroraValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value
            .field
            .double_value
            .ok_or_else(|| Error::Decode("Not a double value".into()))?)
    }
}

impl Type<Aurora> for f32 {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::Double)
    }
}

impl Type<Aurora> for [f32] {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::DoubleArray)
    }
}

impl Type<Aurora> for Vec<f32> {
    fn type_info() -> AuroraTypeInfo {
        <[f32] as Type<Aurora>>::type_info()
    }
}

impl Encode<'_, Aurora> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        <f64 as Encode<Aurora>>::encode_by_ref(&(*self as f64), buf)
    }
}

impl Decode<'_, Aurora> for f32 {
    fn decode(value: AuroraValueRef<'_>) -> Result<Self, BoxDynError> {
        let f = <f64 as Decode<Aurora>>::decode(value)?;

        if f.gt(&(f32::MAX as f64)) {
            Err("value is larger than f32::MAX".into())
        } else {
            Ok(f as f32)
        }
    }
}
