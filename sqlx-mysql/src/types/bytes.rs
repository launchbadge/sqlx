//! Implements [`Type`] for binary strings in MySQL.
//!
//! -   [`&[u8]`][slice]
//! -   [`Vec<u8>`]
//! -   [`bytes::Bytes`] - `Bytes` can return binary data from the column,
//!     without re-allocation. Most useful when reading large blobs from
//!     the connection.
//!

use bytes::Bytes;
use sqlx_core::{decode, encode, Decode, Encode, Type};

use crate::io::MySqlWriteExt;
use crate::type_info::MySqlTypeInfo;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId};

impl Type<MySql> for &'_ [u8] {
    fn type_id() -> MySqlTypeId {
        MySqlTypeId::TEXT
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(ty.id(), MySqlTypeId::TEXT | MySqlTypeId::CHAR | MySqlTypeId::VARCHAR)
    }
}

impl Encode<MySql> for &'_ [u8] {
    fn encode(&self, _: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        out.buffer().write_bytes_lenenc(self);

        Ok(())
    }
}

impl<'r> Decode<'r, MySql> for &'r [u8] {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_bytes()
    }
}

impl Type<MySql> for Vec<u8> {
    fn type_id() -> MySqlTypeId {
        <&[u8] as Type<MySql>>::type_id()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <&[u8] as Type<MySql>>::compatible(ty)
    }
}

impl Encode<MySql> for Vec<u8> {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        <&[u8] as Encode<MySql>>::encode(&self.as_slice(), ty, out)
    }
}

impl<'r> Decode<'r, MySql> for Vec<u8> {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_bytes().map(ToOwned::to_owned)
    }
}

impl Type<MySql> for Bytes {
    fn type_id() -> MySqlTypeId {
        <&[u8] as Type<MySql>>::type_id()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <&[u8] as Type<MySql>>::compatible(ty)
    }
}

impl Encode<MySql> for Bytes {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        <&[u8] as Encode<MySql>>::encode(&&**self, ty, out)
    }
}

impl<'r> Decode<'r, MySql> for Bytes {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_shared_bytes()
    }
}
