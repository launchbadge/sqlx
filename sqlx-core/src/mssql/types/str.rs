use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::io::MsSqlBufMutExt;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{MsSql, MsSqlTypeInfo, MsSqlValueRef};
use crate::types::Type;

impl Type<MsSql> for str {
    fn type_info() -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::NVarChar, 0))
    }
}

impl Type<MsSql> for String {
    fn type_info() -> MsSqlTypeInfo {
        <str as Type<MsSql>>::type_info()
    }
}

impl Encode<'_, MsSql> for &'_ str {
    fn produces(&self) -> MsSqlTypeInfo {
        MsSqlTypeInfo(TypeInfo::new(DataType::NVarChar, (self.len() * 2) as u32))
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.put_utf16_str(self);

        IsNull::No
    }
}

impl Encode<'_, MsSql> for String {
    fn produces(&self) -> MsSqlTypeInfo {
        <&str as Encode<MsSql>>::produces(&self.as_str())
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&str as Encode<MsSql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl Decode<'_, MsSql> for String {
    fn accepts(ty: &MsSqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::NVarChar
                | DataType::NChar
                | DataType::BigVarChar
                | DataType::VarChar
                | DataType::BigChar
                | DataType::Char
        )
    }

    fn decode(value: MsSqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value
            .type_info
            .0
            .encoding()?
            .decode_without_bom_handling(value.as_bytes()?)
            .0
            .into_owned())
    }
}
