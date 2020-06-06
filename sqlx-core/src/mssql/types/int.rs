use byteorder::{ByteOrder, LittleEndian};

use crate::database::{Database, HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{MsSql, MsSqlTypeInfo, MsSqlValueRef};
use crate::types::Type;

impl Type<MsSql> for i8 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::IntN, 1))
    }
}

impl Encode<'_, MsSql> for i8 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MsSql> for i8 {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::TinyInt | DataType::IntN) && ty.0.size == 1
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value.as_bytes()?[0] as i8)
    }
}

impl Type<MsSql> for i16 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::IntN, 2))
    }
}

impl Encode<'_, MsSql> for i16 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MsSql> for i16 {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::SmallInt | DataType::IntN) && ty.0.size == 2
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_i16(value.as_bytes()?))
    }
}

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

impl Type<MsSql> for i64 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::IntN, 8))
    }
}

impl Encode<'_, MsSql> for i64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MsSql> for i64 {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::BigInt | DataType::IntN) && ty.0.size == 8
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_i64(value.as_bytes()?))
    }
}
