use crate::aurora::type_info::AuroraType;
use crate::aurora::{Aurora, AuroraTypeInfo, AuroraValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::types::Type;

use rusoto_rds_data::{Field, SqlParameter};

impl Type<Aurora> for bool {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::Boolean)
    }
}

impl Type<Aurora> for [bool] {
    fn type_info() -> AuroraTypeInfo {
        AuroraTypeInfo(AuroraType::Boolean)
    }
}

impl Type<Aurora> for Vec<bool> {
    fn type_info() -> AuroraTypeInfo {
        <[bool] as Type<Aurora>>::type_info()
    }
}

impl Encode<'_, Aurora> for bool {
    fn encode_by_ref(&self, buf: &mut Vec<SqlParameter>) -> IsNull {
        buf.push(SqlParameter {
            value: Some(Field {
                boolean_value: Some(*self),
                ..Default::default()
            }),
            ..Default::default()
        });

        IsNull::No
    }
}

impl Decode<'_, Aurora> for bool {
    fn decode(value: AuroraValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value
            .field
            .boolean_value
            .ok_or_else(|| Error::Decode("Not a bool value".into()))?)
    }
}
