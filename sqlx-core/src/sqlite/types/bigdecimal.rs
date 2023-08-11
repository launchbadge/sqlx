use crate::bigdecimal::FromPrimitive;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::sqlite::{
    type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef,
};
use crate::types::Type;
use crate::value::ValueRef;
use bigdecimal::BigDecimal;
use std::str::FromStr;

impl Type<Sqlite> for BigDecimal {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Text)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&str as Type<Sqlite>>::compatible(ty)
            || <f64 as Type<Sqlite>>::compatible(ty)
            || <i64 as Type<Sqlite>>::compatible(ty)
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
        let ty = &value.type_info();
        if <i64 as Type<Sqlite>>::compatible(ty) {
            Ok(BigDecimal::from(value.int64()))
        } else if <f64 as Type<Sqlite>>::compatible(ty) {
            Ok(BigDecimal::from_f64(value.double()).ok_or("bad float")?)
        } else if <&str as Type<Sqlite>>::compatible(ty) {
            Ok(BigDecimal::from_str(value.text()?)?)
        } else {
            Err("bad type".into())
        }
    }
}
