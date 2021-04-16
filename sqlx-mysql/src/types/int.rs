use sqlx_core::{decode, encode, Decode, Encode, Type};

use super::uint::decode_int_or_uint;
use crate::type_info::MySqlTypeInfo;
use crate::{MySql, MySqlOutput, MySqlRawValue, MySqlTypeId};

// check that the incoming value is not too large or too small
// to fit into the target SQL type
fn ensure_not_too_large_or_too_small(value: i128, ty: &MySqlTypeInfo) -> Result<(), encode::Error> {
    let (max, min): (i128, i128) = match ty.id() {
        MySqlTypeId::TINYINT => (i8::MAX as _, i8::MIN as _),
        MySqlTypeId::SMALLINT => (i16::MAX as _, i16::MIN as _),
        MySqlTypeId::MEDIUMINT => (0x7F_FF_FF as _, 0x80_00_00 as _),
        MySqlTypeId::INT => (i32::MAX as _, i32::MIN as _),
        MySqlTypeId::BIGINT => (i64::MAX as _, i64::MIN as _),

        MySqlTypeId::TINYINT_UNSIGNED => (u8::MAX as _, u8::MIN as _),
        MySqlTypeId::SMALLINT_UNSIGNED => (u16::MAX as _, u16::MIN as _),
        MySqlTypeId::MEDIUMINT_UNSIGNED => (0xFF_FF_FF as _, 0 as _),
        MySqlTypeId::INT_UNSIGNED => (u32::MAX as _, u32::MIN as _),
        MySqlTypeId::BIGINT_UNSIGNED => (u64::MAX as _, u64::MIN as _),

        // not an integer type, if we got this far its because this is _unchecked
        // just let it through
        _ => {
            return Ok(());
        }
    };

    if value > max {
        return Err(encode::Error::msg(format!(
            "number `{}` too large to fit in SQL type `{}`",
            value,
            ty.name()
        )));
    }

    if value < min {
        return Err(encode::Error::msg(format!(
            "number `{}` too small to fit in SQL type `{}`",
            value,
            ty.name()
        )));
    }

    Ok(())
}

macro_rules! impl_type_int {
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
            fn encode(&self, ty: &MySqlTypeInfo, out: &mut MySqlOutput<'_>) -> encode::Result {
                ensure_not_too_large_or_too_small((*self $(as $real)?).into(), ty)?;

                out.buffer().extend_from_slice(&self.to_le_bytes());

                Ok(encode::IsNull::No)
            }
        }

        impl<'r> Decode<'r, MySql> for $ty {
            fn decode(value: MySqlRawValue<'r>) -> decode::Result<Self> {
                decode_int_or_uint(&value)
            }
        }
    };
}

impl_type_int! { i8 => TINYINT }
impl_type_int! { i16 => SMALLINT }
impl_type_int! { i32 => INT }
impl_type_int! { i64 => BIGINT }
impl_type_int! { i128 => BIGINT }

#[cfg(target_pointer_width = "64")]
impl_type_int! { isize: i64 => BIGINT }

#[cfg(target_pointer_width = "32")]
impl_type_int! { isize: i32 => INT }
