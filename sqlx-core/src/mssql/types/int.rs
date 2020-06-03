use byteorder::{ByteOrder, LittleEndian};

use crate::database::{Database, HasValueRef};
use crate::decode::Decode;
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{MsSql, MsSqlTypeInfo, MsSqlValueRef};
use crate::types::Type;

impl Type<MsSql> for i32 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo { ty: DataType::Int })
    }
}

impl Decode<'_, MsSql> for i32 {
    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_i32(value.as_bytes()?))
    }
}
