use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;
use std::borrow::Cow;

impl Type<Mssql> for [u8] {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::BigVarBinary, 0))
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::VarBinary | DataType::Binary | DataType::BigVarBinary | DataType::BigBinary
        )
    }
}

impl Encode<'_, Mssql> for &'_ [u8] {
    fn produces(&self) -> Option<MssqlTypeInfo> {
        let size = if self.len() <= 8000 {
            u32::try_from(self.len()).unwrap().max(1)
        } else {
            0xFF_FF
        };
        return Some(MssqlTypeInfo(TypeInfo::new(DataType::BigVarBinary, size)));
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend(*self);
        IsNull::No
    }
}

impl<'r> Decode<'r, Mssql> for &'r [u8] {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_bytes()
    }
}

impl Type<Mssql> for Vec<u8> {
    fn type_info() -> MssqlTypeInfo {
        <[u8] as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <&[u8] as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for Vec<u8> {
    fn produces(&self) -> Option<MssqlTypeInfo> {
        <&[u8] as Encode<Mssql>>::produces(&self.as_slice())
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&[u8] as Encode<Mssql>>::encode(&**self, buf)
    }
}

impl Decode<'_, Mssql> for Vec<u8> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        <&[u8] as Decode<Mssql>>::decode(value).map(ToOwned::to_owned)
    }
}

impl<'r> Type<Mssql> for Cow<'r, [u8]> {
    fn type_info() -> MssqlTypeInfo {
        <[u8] as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <&[u8] as Type<Mssql>>::compatible(ty)
    }
}

impl<'r> Encode<'_, Mssql> for Cow<'r, [u8]> {
    fn produces(&self) -> Option<MssqlTypeInfo> {
        <&[u8] as Encode<Mssql>>::produces(&self.as_ref())
    }

    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        <&[u8] as Encode<Mssql>>::encode(&**self, buf)
    }
}

impl<'r> Decode<'r, Mssql> for Cow<'r, [u8]> {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        <&[u8] as Decode<Mssql>>::decode(value).map(Cow::Borrowed)
    }
}
