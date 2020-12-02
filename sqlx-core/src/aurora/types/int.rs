use crate::aurora::type_info::AuroraType;
use crate::aurora::{Aurora, AuroraTypeInfo, AuroraValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::types::Type;

use rusoto_rds_data::{Field, SqlParameter};

impl Type<Aurora> for i64 {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::Long)
    }
}

impl Type<Aurora> for [i64] {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::LongArray)
    }
}

impl Type<Aurora> for Vec<i64> {
    fn type_info() -> AuroraTypeInfo {
        <[i64] as Type<Aurora>>::type_info()
    }
}

impl Encode<'_, Aurora> for i64 {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        buf.push(SqlParameter {
            value: Some(Field {
                long_value: Some(*self),
                ..Default::default()
            }),
            ..Default::default()
        });

        IsNull::No
    }
}

impl Decode<'_, Aurora> for i64 {
    fn decode(value: AuroraValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value
            .field
            .long_value
            .ok_or_else(|| Error::Decode("Not a double value".into()))?)
    }
}
