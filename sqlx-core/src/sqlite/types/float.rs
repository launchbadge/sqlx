use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::sqlite::type_info::DataType;
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteArguments, SqliteTypeInfo, SqliteValueRef};
use crate::types::Type;

impl Type<Sqlite> for f32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Float)
    }
}

impl<'q> Encode<'q, Sqlite> for f32 {
    fn encode_by_ref(&self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values
            .push(SqliteArgumentValue::Double((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for f32 {
    fn accepts(ty: &SqliteTypeInfo) -> bool {
        true
    }

    fn decode(value: SqliteValueRef<'r>) -> Result<f32, BoxDynError> {
        Ok(value.double() as f32)
    }
}

impl Type<Sqlite> for f64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Float)
    }
}

impl<'q> Encode<'q, Sqlite> for f64 {
    fn encode_by_ref(&self, args: &mut SqliteArguments<'q>) -> IsNull {
        args.values.push(SqliteArgumentValue::Double(*self));

        IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for f64 {
    fn accepts(ty: &SqliteTypeInfo) -> bool {
        true
    }

    fn decode(value: SqliteValueRef<'r>) -> Result<f64, BoxDynError> {
        Ok(value.double())
    }
}
