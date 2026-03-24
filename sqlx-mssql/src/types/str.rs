use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

fn str_compatible(ty: &MssqlTypeInfo) -> bool {
    matches!(
        ty.base_name(),
        "NVARCHAR" | "VARCHAR" | "NCHAR" | "CHAR" | "NTEXT" | "TEXT" | "XML"
    )
}

impl Type<Mssql> for str {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("NVARCHAR")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        str_compatible(ty)
    }
}

impl Encode<'_, Mssql> for &'_ str {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::String((*self).to_owned()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Mssql> for &'r str {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_str()
    }
}

impl Type<Mssql> for String {
    fn type_info() -> MssqlTypeInfo {
        <str as Type<Mssql>>::type_info()
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        <str as Type<Mssql>>::compatible(ty)
    }
}

impl Encode<'_, Mssql> for String {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Mssql>>::encode(self.as_str(), buf)
    }
}

impl Decode<'_, Mssql> for String {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        <&str as Decode<Mssql>>::decode(value).map(ToOwned::to_owned)
    }
}

forward_encode_impl!(Arc<str>, &str, Mssql);
forward_encode_impl!(Rc<str>, &str, Mssql);
forward_encode_impl!(Cow<'_, str>, &str, Mssql);
forward_encode_impl!(Box<str>, &str, Mssql);
