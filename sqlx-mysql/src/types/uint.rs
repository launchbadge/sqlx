use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::error::Error as StdError;
use std::str::FromStr;

use bytes::Buf;
use sqlx_core::{decode, encode};
use sqlx_core::{Decode, Encode, Type};

use crate::type_info::MySqlTypeInfo;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlRawValueFormat, MySqlTypeId};

pub(super) fn decode_int_or_uint<T>(value: &MySqlRawValue<'_>) -> decode::Result<T>
where
    T: TryFrom<i64> + TryFrom<u64> + FromStr,
    <T as TryFrom<u64>>::Error: 'static + StdError + Send + Sync,
    <T as TryFrom<i64>>::Error: 'static + StdError + Send + Sync,
    <T as FromStr>::Err: 'static + StdError + Send + Sync,
{
    if value.format() == MySqlRawValueFormat::Text {
        return Ok(value.as_str()?.parse()?);
    }

    let mut bytes = value.as_bytes()?;
    let is_unsigned = value.type_info().id().is_unsigned();
    let size = cmp::min(bytes.len(), 8);

    Ok(if is_unsigned {
        bytes.get_uint_le(size).try_into()?
    } else {
        bytes.get_int_le(size).try_into()?
    })
}

// check that the incoming value is not too large
// to fit into the target SQL type
fn ensure_not_too_large(value: u128, ty: &MySqlTypeInfo) -> encode::Result<()> {
    let max = match ty.id() {
        MySqlTypeId::TINYINT => i8::MAX as _,
        MySqlTypeId::SMALLINT => i16::MAX as _,
        MySqlTypeId::MEDIUMINT => 0x7F_FF_FF as _,
        MySqlTypeId::INT => i32::MAX as _,
        MySqlTypeId::BIGINT => i64::MAX as _,

        MySqlTypeId::TINYINT_UNSIGNED => u8::MAX as _,
        MySqlTypeId::SMALLINT_UNSIGNED => u16::MAX as _,
        MySqlTypeId::MEDIUMINT_UNSIGNED => 0xFF_FF_FF as _,
        MySqlTypeId::INT_UNSIGNED => u32::MAX as _,
        MySqlTypeId::BIGINT_UNSIGNED => u64::MAX as _,

        // not an integer type
        _ => unreachable!(),
    };

    if value > max {
        return Err(encode::Error::msg(format!(
            "number `{}` too large to fit in SQL type `{}`",
            value,
            ty.name()
        )));
    }

    Ok(())
}

macro_rules! impl_type_uint {
    ($ty:ty $(: $real:ty)? => $sql:ident) => {
        impl Type<MySql> for $ty {
            fn type_id() -> MySqlTypeId {
                MySqlTypeId::$sql
            }

            fn compatible(ty: &MySqlTypeInfo) -> bool {
                ty.id().is_integer()
            }
        }

        impl Encode<MySql> for $ty {
            fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result<()> {
                ensure_not_too_large((*self $(as $real)?).into(), ty)?;

                out.buffer().extend_from_slice(&self.to_le_bytes());

                Ok(())
            }
        }

        impl<'r> Decode<'r, MySql> for $ty {
            fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
                decode_int_or_uint(&value)
            }
        }
    };
}

impl_type_uint! { u8 => TINYINT_UNSIGNED }
impl_type_uint! { u16 => SMALLINT_UNSIGNED }
impl_type_uint! { u32 => INT_UNSIGNED }
impl_type_uint! { u64 => BIGINT_UNSIGNED }
impl_type_uint! { u128 => BIGINT_UNSIGNED }

#[cfg(target_pointer_width = "64")]
impl_type_uint! { usize: u64 => BIGINT_UNSIGNED }

#[cfg(target_pointer_width = "32")]
impl_type_uint! { usize: u32 => INT_UNSIGNED }
