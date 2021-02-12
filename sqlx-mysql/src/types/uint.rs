use bytes::Buf;
use sqlx_core::database::HasOutput;
use sqlx_core::{decode, encode};
use sqlx_core::{Database, Decode, Encode, Runtime};

use crate::type_info::MySqlTypeInfo;
use crate::MySqlRawValueFormat::*;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId};

// https://dev.mysql.com/doc/internals/en/binary-protocol-value.html#packet-ProtocolBinary

// TODO: accepts(ty) -> ty.is_integer()
// TODO: compatible(ty) -> ty.is_integer()

impl<Rt: Runtime> Encode<MySql, Rt> for u8 {
    fn encode(&self, _: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        out.buffer().push(*self);

        Ok(())
    }
}

impl<'r, Rt: Runtime> Decode<'r, MySql, Rt> for u8 {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        Ok(match value.format() {
            Binary => value.as_bytes()?.get_u8(),
            Text => value.as_str()?.parse()?,
        })
    }
}
