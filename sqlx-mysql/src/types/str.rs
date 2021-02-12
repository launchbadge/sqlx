use bytes::Buf;
use sqlx_core::database::HasOutput;
use sqlx_core::{decode, encode};
use sqlx_core::{Database, Decode, Encode};

use crate::type_info::MySqlTypeInfo;
use crate::MySqlRawValueFormat::*;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId};

// https://dev.mysql.com/doc/internals/en/binary-protocol-value.html#packet-ProtocolBinary

// TODO: accepts(ty)
// TODO: compatible(ty)

impl Encode<MySql> for str {
    fn encode(&self, _: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        todo!("encode: &str");

        Ok(())
    }
}

impl Encode<MySql> for String {
    fn encode(&self, _: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        todo!("encode: String");

        Ok(())
    }
}

impl<'r> Decode<'r, MySql> for &'r str {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_str()
    }
}

impl<'r> Decode<'r, MySql> for String {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        value.as_str().map(str::to_owned)
    }
}
