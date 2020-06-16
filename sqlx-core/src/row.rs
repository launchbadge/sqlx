use std::fmt::Debug;

use crate::database::{Database, HasValueRef};
use crate::decode::Decode;
use crate::error::{mismatched_types, Error};
use crate::value::ValueRef;

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
    /// Unlike [`get`](#method.get), this method does not check that the type
    /// being returned from the database is compatible with the Rust type and blindly tries
    /// to decode the value.
    ///
    /// # Panics
    ///
    /// Panics if the column does not exist or its value cannot be decoded into the requested type.
    /// See [`try_get_unchecked`](#method.try_get_unchecked) for a non-panicking version.
    ///
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
    ///  * [`ColumnDecode`] if the value could not be decoded into the requested type.
    ///
    /// [`ColumnDecode`]: crate::Error::ColumnDecode
    /// [`ColumnNotFound`]: crate::Error::ColumnNotFound
    /// [`ColumnIndexOutOfBounds`]: crate::Error::ColumnIndexOutOfBounds
    ///
    fn try_get<'r, T, I>(&'r self, index: I) -> Result<T, Error>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>,
    {
        let value = self.try_get_raw(&index)?;

        if !value.is_null() {
            if let Some(actual_ty) = value.type_info() {
                // NOTE: we opt-out of asserting the type equivalency of NULL because of the
                //       high false-positive rate (e.g., `NULL` in Postgres is `TEXT`).
                if !T::accepts(&actual_ty) {
                    return Err(Error::ColumnDecode {
                        index: format!("{:?}", index),
                        source: mismatched_types::<Self::Database, T>(&actual_ty),
                    });
                }
            }
        }

        T::decode(value).map_err(|source| Error::ColumnDecode {
            index: format!("{:?}", index),
            source,
        })
    }

    /// Index into the database row and decode a single value.
    ///
    /// Unlike [`try_get`](#method.try_get), this method does not check that the type
    /// being returned from the database is compatible with the Rust type and blindly tries
    /// to decode the value.
    ///
    /// # Errors
    ///
    ///  * [`ColumnNotFound`] if the column by the given name was not found.
    ///  * [`ColumnIndexOutOfBounds`] if the `usize` index was greater than the number of columns in the row.
    ///  * [`ColumnDecode`] if the value could not be decoded into the requested type.
    ///
    /// [`ColumnDecode`]: crate::Error::ColumnDecode
    /// [`ColumnNotFound`]: crate::Error::ColumnNotFound
    /// [`ColumnIndexOutOfBounds`]: crate::Error::ColumnIndexOutOfBounds
    ///
    #[inline]
    fn try_get_unchecked<'r, T, I>(&'r self, index: I) -> Result<T, Error>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>,
    {
        let value = self.try_get_raw(&index)?;
        T::decode(value).map_err(|source| Error::ColumnDecode {
            index: format!("{:?}", index),
            source,
        })
    }

    /// Index into the database row and decode a single value.
    ///
    /// # Errors
    ///
    ///  * [`ColumnNotFound`] if the column by the given name was not found.
    ///  * [`ColumnIndexOutOfBounds`] if the `usize` index was greater than the number of columns in the row.
    ///
    /// [`ColumnNotFound`]: crate::Error::ColumnNotFound
    /// [`ColumnIndexOutOfBounds`]: crate::Error::ColumnIndexOutOfBounds
    ///
    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<<Self::Database as HasValueRef<'_>>::ValueRef, Error>
    where
        I: ColumnIndex<Self>;
}

// Prevent users from implementing the `Row` trait.
pub(crate) mod private_row {
    pub trait Sealed {}
}
