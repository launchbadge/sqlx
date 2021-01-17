use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{BoxDynError, Error};
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for bool {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::BitN, 1))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::Bit | DataType::BitN)
    }
}

impl Encode<'_, Mssql> for bool {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(if *self { 1 } else { 0 });

        IsNull::No
    }
}

impl Decode<'_, Mssql> for bool {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(*value.as_bytes()?.get(0).ok_or_else(||Error::Protocol("unexpected packet index:0".to_string()))? == 1)
    }
}
