use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::io::MssqlBufMutExt;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for str {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::NVarChar, 0))
    }
}

impl Type<Mssql> for String {
    fn type_info() -> MssqlTypeInfo {
        <str as Type<Mssql>>::type_info()
    }
}

impl Encode<'_, Mssql> for &'_ str {
    fn produces(&self) -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::NVarChar, (self.len() * 2) as u32))
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.put_utf16_str(self);

        IsNull::No
    }
}

impl Encode<'_, Mssql> for String {
    fn produces(&self) -> MssqlTypeInfo {
        <&str as Encode<Mssql>>::produces(&self.as_str())
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&str as Encode<Mssql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl Decode<'_, Mssql> for String {
    fn accepts(ty: &MssqlTypeInfo) -> bool {
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

    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value
            .type_info
            .0
            .encoding()?
            .decode_without_bom_handling(value.as_bytes()?)
            .0
            .into_owned())
    }
}
