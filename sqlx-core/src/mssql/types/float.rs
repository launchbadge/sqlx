use byteorder::{ByteOrder, LittleEndian};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{MsSql, MsSqlTypeInfo, MsSqlValueRef};
use crate::types::Type;

impl Type<MsSql> for f32 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::FloatN, 4))
    }
}

impl Encode<'_, MsSql> for f32 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MsSql> for f32 {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Real | DataType::FloatN) && ty.0.size == 4
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_f32(value.as_bytes()?))
    }
}

impl Type<MsSql> for f64 {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::FloatN, 8))
    }
}

impl Encode<'_, MsSql> for f64 {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(&self.to_le_bytes());

        IsNull::No
    }
}

impl Decode<'_, MsSql> for f64 {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Float | DataType::FloatN) && ty.0.size == 8
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(LittleEndian::read_f64(value.as_bytes()?))
    }
}
