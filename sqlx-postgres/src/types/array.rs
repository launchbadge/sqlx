use sqlx_core::{encode, Arguments, Database, Encode, Type, TypeEncode};

use crate::{PgOutput, PgTypeId, PgTypeInfo, Postgres};
use sqlx_core::database::HasOutput;

/// Marker trait for types which support being wrapped in array in Postgres.
pub trait PgHasArray {
    /// The type ID in Postgres of the array type which has this type as an element.
    const ARRAY_TYPE_ID: PgTypeId;
}

impl<T: PgHasArray> Type<Postgres> for &'_ [T] {
    fn type_id() -> <Postgres as Database>::TypeId
    where
        Self: Sized,
    {
        T::ARRAY_TYPE_ID
    }

    // TODO: check `PgTypeInfo` for array element type and check compatibility of that
    // fn compatible(ty: &<Postgres as Database>::TypeInfo) -> bool
    // where
    //     Self: Sized,
    // {
    // }
}

impl<T: Type<Postgres> + Encode<Postgres>> Encode<Postgres> for &'_ [T] {
    fn encode(
        &self,
        ty: &<Postgres as Database>::TypeInfo,
        out: &mut <Postgres as HasOutput<'_>>::Output,
    ) -> encode::Result {
        encode_array(*self, ty, out)
    }

    fn vector_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn expand_vector<'a>(&'a self, arguments: &mut Arguments<'a, Postgres>) {
        for elem in *self {
            arguments.add(elem);
        }
    }
}

// Vector

impl<T: PgHasArray> Type<Postgres> for Vec<T> {
    fn type_id() -> <Postgres as Database>::TypeId
    where
        Self: Sized,
    {
        <&[T]>::type_id()
    }

    fn compatible(ty: &<Postgres as Database>::TypeInfo) -> bool
    where
        Self: Sized,
    {
        <&[T]>::compatible(ty)
    }
}

impl<T: Type<Postgres> + Encode<Postgres>> Encode<Postgres> for Vec<T> {
    fn encode(
        &self,
        ty: &<Postgres as Database>::TypeInfo,
        out: &mut <Postgres as HasOutput<'_>>::Output,
    ) -> encode::Result {
        encode_array(self.iter(), ty, out)
    }

    fn vector_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn expand_vector<'a>(&'a self, arguments: &mut Arguments<'a, Postgres>) {
        for elem in self {
            arguments.add(elem);
        }
    }
}

// static-size arrays

impl<T: PgHasArray, const N: usize> Type<Postgres> for [T; N] {
    fn type_id() -> <Postgres as Database>::TypeId
    where
        Self: Sized,
    {
        <&[T]>::type_id()
    }

    fn compatible(ty: &<Postgres as Database>::TypeInfo) -> bool
    where
        Self: Sized,
    {
        <&[T]>::compatible(ty)
    }
}

impl<T: Type<Postgres> + Encode<Postgres>, const N: usize> Encode<Postgres> for [T; N] {
    fn encode(
        &self,
        ty: &<Postgres as Database>::TypeInfo,
        out: &mut <Postgres as HasOutput<'_>>::Output,
    ) -> encode::Result {
        encode_array(self.iter(), ty, out)
    }

    fn vector_len(&self) -> Option<usize> {
        Some(self.len())
    }

    fn expand_vector<'a>(&'a self, arguments: &mut Arguments<'a, Postgres>) {
        for elem in self {
            arguments.add(elem);
        }
    }
}

pub fn encode_array<T: TypeEncode<Postgres>, I: IntoIterator<Item = T>>(
    array: I,
    _ty: &PgTypeInfo,
    out: &mut PgOutput<'_>,
) -> encode::Result {
    // number of dimensions (1 for now)
    out.buffer().extend_from_slice(&1i32.to_be_bytes());

    let len_start = out.buffer().len();

    // whether or not the array is null (fixup afterward)
    out.buffer().extend_from_slice(&[0; 4]);

    // FIXME: better error message/avoid the error
    let elem_type = T::type_id().oid().ok_or_else(|| {
        encode::Error::msg("can only bind an array with elements with a known oid")
    })?;

    out.buffer().extend_from_slice(&elem_type.to_be_bytes());

    let mut count: i32 = 0;

    let is_null = array
        .into_iter()
        .map(|elem| {
            count = count
                .checked_add(1)
                .ok_or_else(|| encode::Error::msg("array length overflows i32"))?;
            elem.encode(&PgTypeInfo(T::type_id()), out)
        })
        .collect::<encode::Result>()?;

    // fixup the length
    out.buffer()[len_start..][..4].copy_from_slice(&count.to_be_bytes());

    Ok(is_null)
}
