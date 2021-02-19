use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::error::Error as StdError;
use std::num::TryFromIntError;
use std::str::FromStr;

use bytes::Buf;
use sqlx_core::{decode, encode, Database, TypeEncode};
use sqlx_core::{Decode, Encode, Type};

use crate::type_info::MySqlTypeInfo;
use crate::MySqlRawValueFormat::*;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlRawValueFormat, MySqlTypeId};

// https://dev.mysql.com/doc/internals/en/binary-protocol-value.html#packet-ProtocolBinary

const NUMBER_TOO_LARGE: &str = "number too large to fit in target type";

// shared among all Decode impls for unsigned and signed integers
fn decode_int_or_uint<T>(value: MySqlRawValue<'_>) -> decode::Result<T>
where
    T: TryFrom<i64> + TryFrom<u64> + FromStr,
    <T as TryFrom<i64>>::Error: 'static + StdError + Send + Sync,
    <T as TryFrom<u64>>::Error: 'static + StdError + Send + Sync,
    <T as FromStr>::Err: 'static + StdError + Send + Sync,
{
    if value.format() == MySqlRawValueFormat::Text {
        return Ok(value.as_str()?.parse()?);
    }

    let mut bytes = value.as_bytes()?;

    // start from u64 if the value is marked as unsigned
    // otherwise start from i64
    let is_unsigned = value.type_info().id().is_unsigned();

    // pull at most 8 bytes from the buffer
    let len = cmp::max(bytes.len(), 8);

    Ok(if is_unsigned {
        bytes.get_uint_le(len).try_into()?
    } else {
        bytes.get_int_le(len).try_into()?
    })
}

impl Type<MySql> for u8 {
    fn type_id() -> MySqlTypeId {
        MySqlTypeId::TINYINT_UNSIGNED
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        ty.id().is_integer()
    }
}

impl TypeEncode<MySql> for u8 {
    fn type_id(&self, _: &MySqlTypeInfo) -> MySqlTypeId {
        <u8 as Type<MySql>>::type_id()
    }
}

impl Encode<MySql> for u8 {
    fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
        match ty.id() {
            MySqlTypeId::TINYINT_UNSIGNED => {}

            MySqlTypeId::TINYINT if *self > 0x7f => {
                return Err(encode::Error::msg(NUMBER_TOO_LARGE));
            }

            _ => {}
        }

        out.buffer().push(*self);

        Ok(())
    }
}

impl<'r> Decode<'r, MySql> for u8 {
    fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
        decode_int_or_uint(value)
    }
}
