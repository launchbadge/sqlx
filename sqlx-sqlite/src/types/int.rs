use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};

impl Type<Sqlite> for i8 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, Sqlite> for i8 {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Int(*self as i32));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for i8 {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int().try_into()?)
    }
}

impl Type<Sqlite> for i16 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, Sqlite> for i16 {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Int(*self as i32));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for i16 {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int().try_into()?)
    }
}

impl Type<Sqlite> for i32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Int)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, Sqlite> for i32 {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Int(*self));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for i32 {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int())
    }
}

impl Type<Sqlite> for i64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Int64)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, Sqlite> for i64 {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        args.push(SqliteArgumentValue::Int64(*self));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for i64 {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(value.int64())
    }
}
