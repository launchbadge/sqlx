use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::sqlite::{
    type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef,
};
use crate::types::Type;
use bigdecimal::BigDecimal;

impl Type<Sqlite> for BigDecimal {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Text)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
    }
}

impl Encode<'_, Sqlite> for BigDecimal {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let string_value = self.to_string();
        Encode::<Sqlite>::encode(string_value, buf)
    }
}

impl Decode<'_, Sqlite> for BigDecimal {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let string_value = <&str as Decode<Sqlite>>::decode(value)?;

        string_value.parse().map_err(Into::into)
    }
}
