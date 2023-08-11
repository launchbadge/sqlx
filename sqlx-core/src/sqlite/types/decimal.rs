use bigdecimal_::FromPrimitive;
use rust_decimal::Decimal;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::sqlite::{
    type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef,
};
use crate::types::Type;

impl Type<Sqlite> for Decimal {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Text)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
            || <f64 as Type<Sqlite>>::compatible(ty)
            || <i64 as Type<Sqlite>>::compatible(ty)
    }
}

impl Encode<'_, Sqlite> for Decimal {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let string_value = self.to_string();
        Encode::<Sqlite>::encode(string_value, buf)
    }
}

impl Decode<'_, Sqlite> for Decimal {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.type_info().0 {
            DataType::Float => Ok(Decimal::from_f64(value.double()).ok_or("bad float")?),
            DataType::Int | DataType::Int64 => Ok(Decimal::from(value.int64())),
            _ => {
                let string_value = <&str as Decode<Sqlite>>::decode(value)?;
                string_value.parse().map_err(Into::into)
            }
        }
    }
}
