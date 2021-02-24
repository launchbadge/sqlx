use std::str::from_utf8_unchecked;

use bytes::Bytes;
use bytestring::ByteString;
use sqlx_core::{decode, encode, Type};
use sqlx_core::{Decode, Encode};

use crate::io::MySqlWriteExt;
use crate::type_info::MySqlTypeInfo;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId};

impl Type<MySql> for &'_ str {
    fn type_id() -> MySqlTypeId {
        MySqlTypeId::TEXT
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(ty.id(), MySqlTypeId::TEXT | MySqlTypeId::CHAR | MySqlTypeId::VARCHAR)
    }
}

impl Encode<MySql> for &'_ str {
    fn encode(&self, _: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        out.buffer().write_bytes_lenenc(self.as_bytes());

        Ok(())
    }
}

impl<'r> Decode<'r, MySql> for &'r str {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_str()
    }
}

impl Type<MySql> for String {
    fn type_id() -> MySqlTypeId {
        <&str as Type<MySql>>::type_id()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <&str as Type<MySql>>::compatible(ty)
    }
}

impl Encode<MySql> for String {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        <&str as Encode<MySql>>::encode(&self.as_str(), ty, out)
    }
}

impl<'r> Decode<'r, MySql> for String {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_str().map(str::to_owned)
    }
}

impl Type<MySql> for ByteString {
    fn type_id() -> MySqlTypeId {
        <&str as Type<MySql>>::type_id()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <&str as Type<MySql>>::compatible(ty)
    }
}

impl Encode<MySql> for ByteString {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        <&str as Encode<MySql>>::encode(&&**self, ty, out)
    }
}

impl<'r> Decode<'r, MySql> for ByteString {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_shared_str()
    }
}
