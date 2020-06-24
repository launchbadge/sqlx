use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mysql::protocol::text::ColumnType;
use crate::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
use crate::types::Type;

impl Type<MySql> for bool {
    fn type_info() -> MySqlTypeInfo {
        // MySQL has no actual `BOOLEAN` type, the type is an alias of `TINYINT(1)`
        <i8 as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        let is_one_bit = ty.r#type == ColumnType::Bit && ty.max_size == Some(1);
        <i8 as Type<MySql>>::compatible(ty) || is_one_bit
    }
}

impl Encode<'_, MySql> for bool {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <i8 as Encode<MySql>>::encode(*self as i8, buf)
    }
}

impl Decode<'_, MySql> for bool {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(<i8 as Decode<MySql>>::decode(value)? != 0)
    }
}
