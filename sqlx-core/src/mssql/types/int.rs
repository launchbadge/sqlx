use byteorder::{ByteOrder, LittleEndian};

use crate::database::{Database, HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{MsSql, MsSqlTypeInfo, MsSqlValueRef};
use crate::types::Type;

impl Type<MsSql> for i32 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::IntN, 4))
    }
}

impl Encode<'_, MsSql> for i32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MsSql> for i32 {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Int | DataType::IntN) && ty.0.size == 4
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_i32(value.as_bytes()?))
    }
}
