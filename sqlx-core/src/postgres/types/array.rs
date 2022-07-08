use bytes::Buf;
use std::borrow::Cow;

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::type_info::PgType;
use crate::postgres::types::Oid;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

pub trait PgHasArrayType {
    fn array_type_info() -> PgTypeInfo;
    fn array_compatible(ty: &PgTypeInfo) -> bool {
        *ty == Self::array_type_info()
    }
}

impl<T> PgHasArrayType for Option<T>
where
    T: PgHasArrayType,
{
    fn array_type_info() -> PgTypeInfo {
        T::array_type_info()
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        T::array_compatible(ty)
    }
}

impl<T> Type<Postgres> for [T]
where
    T: PgHasArrayType,
{
    fn type_info() -> PgTypeInfo {
        T::array_type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        T::array_compatible(ty)
    }
}

impl<T> Type<Postgres> for Vec<T>
where
    T: PgHasArrayType,
{
    fn type_info() -> PgTypeInfo {
        T::array_type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        T::array_compatible(ty)
    }
}

impl<T, const N: usize> Type<Postgres> for [T; N]
where
    T: PgHasArrayType,
{
    fn type_info() -> PgTypeInfo {
        T::array_type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        T::array_compatible(ty)
    }
}

impl<'q, T> Encode<'q, Postgres> for Vec<T>
where
    for<'a> &'a [T]: Encode<'q, Postgres>,
    T: Encode<'q, Postgres>,
{
    #[inline]
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.as_slice().encode_by_ref(buf)
    }
}

impl<'q, T, const N: usize> Encode<'q, Postgres> for [T; N]
where
    for<'a> &'a [T]: Encode<'q, Postgres>,
    T: Encode<'q, Postgres>,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        self.as_slice().encode_by_ref(buf)
    }
}

impl<'q, T> Encode<'q, Postgres> for &'_ [T]
where
    T: Encode<'q, Postgres> + Type<Postgres>,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let type_info = if self.len() < 1 {
            T::type_info()
        } else {
            self[0].produces().unwrap_or_else(T::type_info)
        };

        buf.extend(&1_i32.to_be_bytes()); // number of dimensions
        buf.extend(&0_i32.to_be_bytes()); // flags

        // element type
        match type_info.0 {
            PgType::DeclareWithName(name) => buf.patch_type_by_name(&name),

            ty => {
                buf.extend(&ty.oid().0.to_be_bytes());
            }
        }

        buf.extend(&(self.len() as i32).to_be_bytes()); // len
        buf.extend(&1_i32.to_be_bytes()); // lower bound

        for element in self.iter() {
            buf.encode(element);
        }

        IsNull::No
    }
}

impl<'r, T, const N: usize> Decode<'r, Postgres> for [T; N]
where
    T: for<'a> Decode<'a, Postgres> + Type<Postgres>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        // This could be done more efficiently by refactoring the Vec decoding below so that it can
        // be used for arrays and Vec.
        let vec: Vec<T> = Decode::decode(value)?;
        let array: [T; N] = vec.try_into().map_err(|_| "wrong number of elements")?;
        Ok(array)
    }
}

impl<'r, T> Decode<'r, Postgres> for Vec<T>
where
    T: for<'a> Decode<'a, Postgres> + Type<Postgres>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let format = value.format();

        match format {
            PgValueFormat::Binary => {
                // https://github.com/postgres/postgres/blob/a995b371ae29de2d38c4b7881cf414b1560e9746/src/backend/utils/adt/arrayfuncs.c#L1548

                let mut buf = value.as_bytes()?;

                // number of dimensions in the array
                let ndim = buf.get_i32();

                if ndim == 0 {
                    // zero dimensions is an empty array
                    return Ok(Vec::new());
                }

                if ndim != 1 {
                    return Err(format!("encountered an array of {} dimensions; only one-dimensional arrays are supported", ndim).into());
                }

                // appears to have been used in the past to communicate potential NULLS
                // but reading source code back through our supported postgres versions (9.5+)
                // this is never used for anything
                let _flags = buf.get_i32();

                // the OID of the element
                let element_type_oid = Oid(buf.get_u32());
                let element_type_info: PgTypeInfo = PgTypeInfo::try_from_oid(element_type_oid)
                    .or_else(|| value.type_info.try_array_element().map(Cow::into_owned))
                    .ok_or_else(|| {
                        BoxDynError::from(format!(
                            "failed to resolve array element type for oid {}",
                            element_type_oid.0
                        ))
                    })?;

                // length of the array axis
                let len = buf.get_i32();

                // the lower bound, we only support arrays starting from "1"
                let lower = buf.get_i32();

                if lower != 1 {
                    return Err(format!("encountered an array with a lower bound of {} in the first dimension; only arrays starting at one are supported", lower).into());
                }

                let mut elements = Vec::with_capacity(len as usize);

                for _ in 0..len {
                    elements.push(T::decode(PgValueRef::get(
                        &mut buf,
                        format,
                        element_type_info.clone(),
                    ))?)
                }

                Ok(elements)
            }

            PgValueFormat::Text => {
                // no type is provided from the database for the element
                let element_type_info = T::type_info();

                let s = value.as_str()?;

                // https://github.com/postgres/postgres/blob/a995b371ae29de2d38c4b7881cf414b1560e9746/src/backend/utils/adt/arrayfuncs.c#L718

                // trim the wrapping braces
                let s = &s[1..(s.len() - 1)];

                if s.is_empty() {
                    // short-circuit empty arrays up here
                    return Ok(Vec::new());
                }

                // NOTE: Nearly *all* types use ',' as the sequence delimiter. Yes, there is one
                //       that does not. The BOX (not PostGIS) type uses ';' as a delimiter.

                // TODO: When we add support for BOX we need to figure out some way to make the
                //       delimiter selection

                let delimiter = ',';
                let mut done = false;
                let mut in_quotes = false;
                let mut in_escape = false;
                let mut value = String::with_capacity(10);
                let mut chars = s.chars();
                let mut elements = Vec::with_capacity(4);

                while !done {
                    loop {
                        match chars.next() {
                            Some(ch) => match ch {
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
                            },

                            None => {
                                done = true;
                                break;
                            }
                        }
                    }

                    let value_opt = if value == "NULL" {
                        None
                    } else {
                        Some(value.as_bytes())
                    };

                    elements.push(T::decode(PgValueRef {
                        value: value_opt,
                        row: None,
                        type_info: element_type_info.clone(),
                        format,
                    })?);

                    value.clear();
                }

                Ok(elements)
            }
        }
    }
}
