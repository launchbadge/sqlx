use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::error::Error as StdError;
use std::str::FromStr;

use bytes::Buf;
use sqlx_core::{decode, encode, Decode, Encode, Type};

use crate::{PgOutput, PgRawValue, PgRawValueFormat, PgTypeId, PgTypeInfo, Postgres};

// https://www.postgresql.org/docs/current/datatype-numeric.html#DATATYPE-INT

// check that the incoming value is not too large or too small
// to fit into the target SQL type
fn ensure_not_too_large_or_too_small(value: i128, ty: &PgTypeInfo) -> Result<(), encode::Error> {
    let (max, min): (i128, i128) = match ty.id() {
        PgTypeId::SMALLINT => (i16::MAX as _, i16::MIN as _),
        PgTypeId::INTEGER => (i32::MAX as _, i32::MIN as _),
        PgTypeId::BIGINT => (i64::MAX as _, i64::MIN as _),

        _ => {
            // for non-integer types, ignore the check
            // if we got this far its because someone asked for and `_unchecked` bind
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

fn ensure_not_too_large(value: u128, ty: &PgTypeInfo) -> Result<(), encode::Error> {
    let max: u128 = match ty.id() {
        PgTypeId::SMALLINT => i16::MAX as _,
        PgTypeId::INTEGER => i32::MAX as _,
        PgTypeId::BIGINT => i64::MAX as _,

        _ => {
            // for non-integer types, ignore the check
            // if we got this far its because someone asked for and `_unchecked` bind
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

    Ok(())
}

fn decode_int<T>(value: &PgRawValue<'_>) -> decode::Result<T>
where
    T: TryFrom<i64> + TryFrom<u64> + FromStr,
    <T as TryFrom<u64>>::Error: 'static + StdError + Send + Sync,
    <T as TryFrom<i64>>::Error: 'static + StdError + Send + Sync,
    <T as FromStr>::Err: 'static + StdError + Send + Sync,
{
    if value.format() == PgRawValueFormat::Text {
        return Ok(value.as_str()?.parse()?);
    }

    let mut bytes = value.as_bytes()?;
    let size = cmp::min(bytes.len(), 8);

    Ok(bytes.get_int(size).try_into()?)
}

macro_rules! impl_type_int {
    ($ty:ty $(: $real:ty)? => $sql:ident $(, [] => $array_sql:ident)?) => {
        impl Type<Postgres> for $ty {
            fn type_id() -> PgTypeId {
                PgTypeId::$sql
            }

            fn compatible(ty: &PgTypeInfo) -> bool {
                ty.id().is_integer()
            }
        }

        $(
            impl super::array::PgHasArray for $ty {
                const ARRAY_TYPE_ID: PgTypeId = PgTypeId::$array_sql;
            }
        )?

        impl Encode<Postgres> for $ty {
            fn encode(&self, ty: &PgTypeInfo, out: &mut PgOutput<'_>) -> encode::Result {
                ensure_not_too_large_or_too_small((*self $(as $real)?).into(), ty)?;

                out.buffer().extend_from_slice(&self.to_be_bytes());

                Ok(encode::IsNull::No)
            }
        }

        impl<'r> Decode<'r, Postgres> for $ty {
            fn decode(value: PgRawValue<'r>) -> decode::Result<Self> {
                decode_int(&value)
            }
        }
    };
}

impl_type_int! { i8 => SMALLINT, [] => SMALLINT_ARRAY }
impl_type_int! { i16 => SMALLINT, [] => SMALLINT_ARRAY }
impl_type_int! { i32 => INTEGER, [] => INTEGER_ARRAY }
impl_type_int! { i64 => BIGINT }
impl_type_int! { i128 => BIGINT }

macro_rules! impl_type_uint {
    ($ty:ty $(: $real:ty)? => $sql:ident) => {
        impl Type<Postgres> for $ty {
            fn type_id() -> PgTypeId {
                PgTypeId::$sql
            }

            fn compatible(ty: &PgTypeInfo) -> bool {
                ty.id().is_integer()
            }
        }

        impl Encode<Postgres> for $ty {
            fn encode(&self, ty: &PgTypeInfo, out: &mut PgOutput<'_>) -> encode::Result {
                ensure_not_too_large((*self $(as $real)?).into(), ty)?;

                out.buffer().extend_from_slice(&self.to_be_bytes());

                Ok(encode::IsNull::No)
            }
        }

        impl<'r> Decode<'r, Postgres> for $ty {
            fn decode(value: PgRawValue<'r>) -> decode::Result<Self> {
                decode_int(&value)
            }
        }
    };
}

impl_type_uint! { u8 => SMALLINT }
impl_type_uint! { u16 => SMALLINT }
impl_type_uint! { u32 => INTEGER }
impl_type_uint! { u64 => BIGINT }
impl_type_uint! { u128 => BIGINT }
