use bytes::Buf;

use crate::database::{HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::type_info::PgType;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;
use std::iter::FromIterator;

impl<T> Type<Postgres> for [Option<T>]
where
    [T]: Type<Postgres>,
{
    fn type_info() -> PgTypeInfo {
        <[T] as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <[T] as Type<Postgres>>::compatible(ty)
    }
}

impl<T> Type<Postgres> for Vec<Option<T>>
where
    Vec<T>: Type<Postgres>,
{
    fn type_info() -> PgTypeInfo {
        <Vec<T> as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <Vec<T> as Type<Postgres>>::compatible(ty)
    }
}

impl<I> Type<Postgres> for crate::types::Array<I>
where
    I: IntoIterator,
    [I::Item]: Type<Postgres>,
{
    fn type_info() -> PgTypeInfo {
        <[I::Item] as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <[I::Item] as Type<Postgres>>::compatible(ty)
    }
}

impl<'q, T> Encode<'q, Postgres> for Vec<T>
where
    T: Encode<'q, Postgres> + Type<Postgres>,
{
    #[inline]
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.encode_iter(self.as_slice());
        IsNull::No
    }
}

impl<'q, T> Encode<'q, Postgres> for &'_ [T]
where
    T: Encode<'q, Postgres> + Type<Postgres>,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.encode_iter(*self);
        IsNull::No
    }
}

impl<'q, T, I> Encode<'q, Postgres> for crate::types::Array<I>
where
    for<'a> &'a I: IntoIterator<Item = T>,
    T: Encode<'q, Postgres> + Type<Postgres>,
{
    fn encode_by_ref(&self, buf: &mut <Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        buf.encode_iter(&self.0);
        IsNull::No
    }
}

impl<'r, T> Decode<'r, Postgres> for Vec<T>
where
    T: for<'a> Decode<'a, Postgres> + Type<Postgres>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        // `impl<T> FromIterator<T> for Vec<T>` is specialized for `vec::IntoIter<T>`:
        // https://github.com/rust-lang/rust/blob/8a9fa3682dcf0de095ec308a29a7b19b0e011ef0/library/alloc/src/vec/spec_from_iter.rs
        decode_array(value)
    }
}

impl<'r, I> Decode<'r, Postgres> for crate::types::Array<I>
where
    I: IntoIterator + FromIterator<<I as IntoIterator>::Item>,
    I::Item: for<'a> Decode<'a, Postgres> + Type<Postgres>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        decode_array(value).map(Self)
    }
}

fn decode_array<T, I>(value: PgValueRef<'_>) -> Result<I, BoxDynError>
where
    I: FromIterator<T>,
    T: for<'a> Decode<'a, Postgres> + Type<Postgres>,
{
    let element_type_info;
    let format = value.format();

    match format {
        PgValueFormat::Binary => {
            // https://github.com/postgres/postgres/blob/a995b371ae29de2d38c4b7881cf414b1560e9746/src/backend/utils/adt/arrayfuncs.c#L1548

            let mut buf = value.as_bytes()?;

            // number of dimensions in the array
            let ndim = buf.get_i32();

            if ndim == 0 {
                // zero dimensions is an empty array
                return Ok(I::from_iter(std::iter::empty()));
            }

            if ndim != 1 {
                return Err(format!("encountered an array of {} dimensions; only one-dimensional arrays are supported", ndim).into());
            }

            // appears to have been used in the past to communicate potential NULLS
            // but reading source code back through our supported postgres versions (9.5+)
            // this is never used for anything
            let _flags = buf.get_i32();

            // the OID of the element
            let element_type_oid = buf.get_u32();
            element_type_info = PgTypeInfo::try_from_oid(element_type_oid)
                .unwrap_or_else(|| PgTypeInfo(PgType::DeclareWithOid(element_type_oid)));

            // length of the array axis
            let len = buf.get_i32();

            // the lower bound, we only support arrays starting from "1"
            let lower = buf.get_i32();

            if lower != 1 {
                return Err(format!("encountered an array with a lower bound of {} in the first dimension; only arrays starting at one are supported", lower).into());
            }

            (0..len)
                .map(|_| T::decode(PgValueRef::get(&mut buf, format, element_type_info.clone())))
                .collect()
        }

        PgValueFormat::Text => {
            // no type is provided from the database for the element
            element_type_info = T::type_info();

            let s = value.as_str()?;

            // https://github.com/postgres/postgres/blob/a995b371ae29de2d38c4b7881cf414b1560e9746/src/backend/utils/adt/arrayfuncs.c#L718

            // trim the wrapping braces
            let s = &s[1..(s.len() - 1)];

            if s.is_empty() {
                // short-circuit empty arrays up here
                return Ok(I::from_iter(std::iter::empty()));
            }

            // NOTE: Nearly *all* types use ',' as the sequence delimiter. Yes, there is one
            //       that does not. The BOX (not PostGIS) type uses ';' as a delimiter.

            // TODO: When we add support for BOX we need to figure out some way to make the
            //       delimiter selection

            let delimiter = ',';
            let mut in_quotes = false;
            let mut in_escape = false;
            let mut value = String::with_capacity(10);
            let mut chars = s.chars();

            std::iter::from_fn(|| {
                if chars.as_str().is_empty() {
                    return None;
                }

                for ch in &mut chars {
                    match ch {
                        _ if in_escape => {
                            value.push(ch);
                            in_escape = false;
                        }

                        '"' => {
                            in_quotes = !in_quotes;
                        }

                        '\\' => {
                            in_escape = true;
                        }

                        _ if ch == delimiter && !in_quotes => {
                            break;
                        }

                        _ => {
                            value.push(ch);
                        }
                    }
                }

                let value_opt = if value == "NULL" {
                    None
                } else {
                    Some(value.as_bytes())
                };

                let ret = T::decode(PgValueRef {
                    value: value_opt,
                    row: None,
                    type_info: element_type_info.clone(),
                    format,
                });

                value.clear();

                Some(ret)
            })
            .collect()
        }
    }
}
