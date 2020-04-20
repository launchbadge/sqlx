//! Contains the `ColumnIndex`, `Row`, and `FromRow` traits.

use std::fmt::Debug;

use crate::database::{Database, HasRawValue};
use crate::decode::Decode;
use crate::error::Error;

/// A type that can be used to index into a [`Row`].
///
/// The [`get`] and [`try_get`] methods of [`Row`] accept any type that implements `ColumnIndex`.
/// This trait is implemented for strings which are used to look up a column by name, and for
/// `usize` which is used as a positional index into the row.
///
/// This trait is sealed and cannot be implemented for types outside of SQLx.
///
/// [`Row`]: trait.Row.html
/// [`get`]: trait.Row.html#method.get
/// [`try_get`]: trait.Row.html#method.try_get
pub trait ColumnIndex<R: Row + ?Sized>: private_column_index::Sealed + Debug {
    /// Returns a valid positional index into the row, [`ColumnIndexOutOfBounds`], or,
    /// [`ColumnNotFound`].
    ///
    /// [`ColumnNotFound`]: ../enum.Error.html#variant.ColumnNotFound
    /// [`ColumnIndexOutOfBounds`]: ../enum.Error.html#variant.ColumnIndexOutOfBounds
    fn index(&self, row: &R) -> Result<usize, Error>;
}

impl<R, I> ColumnIndex<R> for &'_ I
where
    R: Row + ?Sized,
    I: ColumnIndex<R> + ?Sized,
{
    #[inline]
    fn index(&self, row: &R) -> Result<usize, Error> {
        (**self).index(row)
    }
}

impl<R: Row> ColumnIndex<R> for usize {
    fn index(&self, row: &R) -> Result<usize, Error> {
        let len = row.len();

        if *self >= len {
            return Err(Error::ColumnIndexOutOfBounds { len, index: *self });
        }

        Ok(*self)
    }
}

// Prevent users from implementing the `ColumnIndex` trait.
mod private_column_index {
    pub trait Sealed {}

    impl Sealed for usize {}
    impl Sealed for str {}
    impl<T> Sealed for &'_ T where T: Sealed + ?Sized {}
}

/// Represents a single row from the database.
///
/// This trait is sealed and cannot be implemented for types outside of SQLx.
///
/// [`FromRow`]: crate::row::FromRow
/// [`Cursor`]: crate::cursor::Cursor
/// [`Query::fetch`]: crate::query::Query::fetch
pub trait Row: private_row::Sealed + Unpin + Send + Sync + 'static {
    type Database: Database;

    /// Returns `true` if this row has no columns.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of columns in this row.
    fn len(&self) -> usize;

    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name and a `usize` index
    /// can be used to access a column by position.
    ///
    /// # Panics
    ///
    /// Panics if the column does not exist or its value cannot be decoded into the requested type.
    /// See [`try_get`](#method.try_get) for a non-panicking version.
    ///
    #[inline]
    fn get<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>,
    {
        self.try_get::<T, I>(index).unwrap()
    }

    /// Index into the database row and decode a single value.
    ///
    /// See [`try_get_unchecked`](#method.try_get_unchecked).
    #[inline]
    fn get_unchecked<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>,
    {
        self.try_get_unchecked::<T, I>(index).unwrap()
    }

    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name and a `usize` index
    /// can be used to access a column by position.
    ///
    /// # Errors
    ///
    ///  * [`ColumnNotFound`] if the column by the given name was not found.
    ///  * [`ColumnIndexOutOfBounds`] if the `usize` index was greater than the number of columns in the row.
    ///  * [`Decode`] if the value could not be decoded into the requested type.
    ///
    /// [`Decode`]: crate::Error::Decode
    /// [`ColumnNotFound`]: crate::Error::ColumnNotFound
    /// [`ColumnIndexOutOfBounds`]: crate::Error::ColumnIndexOutOfBounds
    fn try_get<'r, T, I>(&'r self, index: I) -> Result<T, Error>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>,
    {
        let value = self.try_get_raw(&index)?;

        // TODO: Check!

        // if let Some(expected_ty) = value.type_info() {
        //     // NOTE: If there is no type, the value is NULL. This is fine. If the user tries
        //     //       to get this into a non-Option we catch that elsewhere and report as
        //     //       UnexpectedNullError.
        //
        //     if !expected_ty.compatible(&T::type_info()) {
        //         return Err(crate::Error::mismatched_types::<Self::Database, T>(
        //             expected_ty,
        //         ));
        //     }
        // }

        T::decode(value).map_err(|cause| Error::Decode {
            index: format!("{:?}", index),
            cause,
        })
    }

    /// Index into the database row and decode a single value.
    ///
    /// Unlike [`try_get`](#method.try_get), this method does not check that the type
    /// being returned from the database is compatible with the Rust type and just blindly tries
    /// to decode the value. An example of where this could be useful is decoding a Postgres
    /// enumeration as a Rust string (instead of deriving a new Rust enum).
    #[inline]
    fn try_get_unchecked<'r, T, I>(&'r self, index: I) -> Result<T, Error>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>,
    {
        self.try_get_raw(&index).and_then(|value| {
            T::decode(value).map_err(|cause| Error::Decode {
                index: format!("{:?}", index),
                cause,
            })
        })
    }

    // noinspection RsNeedlessLifetimes
    fn try_get_raw<'r, I>(
        &'r self,
        index: I,
    ) -> Result<<Self::Database as HasRawValue<'r>>::RawValue, Error>
    where
        I: ColumnIndex<Self>;
}

// Prevent users from implementing the `Row` trait.
pub(crate) mod private_row {
    pub trait Sealed {}
}

/// A record that can be built from a row returned by the database.
///
/// In order to use [`query_as`] the output type must implement `FromRow`.
///
/// # Deriving
///
/// This trait can be automatically derived by SQLx for any struct. The generated implementation
/// will consist of a sequence of calls to [`Row::try_get`] using the name from each
/// struct field.
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// struct User {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// [`query_as`]: crate::query_as
/// [`Row::try_get`]: crate::row::Row::try_get
pub trait FromRow<'r, R: Row>: Sized {
    fn from_row(row: &'r R) -> Result<Self, Error>;
}

// implement FromRow for tuples of types that implement Decode
// up to tuples of 9 values

macro_rules! impl_from_row_for_tuple {
    ($( ($idx:tt) -> $T:ident );+;) => {
        impl<'r, R, $($T,)+> crate::row::FromRow<'r, R> for ($($T,)+)
        where
            R: Row,
            $($T: crate::decode::Decode<'r, R::Database>,)+
        {
            #[inline]
            fn from_row(row: &'r R) -> Result<Self, Error> {
                use crate::row::Row;

                Ok(($(row.try_get($idx as usize)?,)+))
            }
        }
    };
}

impl_from_row_for_tuple!(
    (0) -> T1;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
);
