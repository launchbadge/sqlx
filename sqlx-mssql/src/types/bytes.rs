use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

fn bytes_compatible(ty: &MssqlTypeInfo) -> bool {
    matches!(ty.base_name(), "VARBINARY" | "BINARY" | "IMAGE")
}

impl Type<Mssql> for [u8] {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("VARBINARY")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        bytes_compatible(ty)
    }
}

impl Encode<'_, Mssql> for &'_ [u8] {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::Binary(self.to_vec()));
        Ok(IsNull::No)
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
        <[u8] as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for Vec<u8> {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        <&[u8] as Encode<Mssql>>::encode(&**self, buf)
    }
}

impl Decode<'_, Mssql> for Vec<u8> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        <&[u8] as Decode<Mssql>>::decode(value).map(ToOwned::to_owned)
    }
}

forward_encode_impl!(Arc<[u8]>, &[u8], Mssql);
forward_encode_impl!(Rc<[u8]>, &[u8], Mssql);
forward_encode_impl!(Box<[u8]>, &[u8], Mssql);
forward_encode_impl!(Cow<'_, [u8]>, &[u8], Mssql);
