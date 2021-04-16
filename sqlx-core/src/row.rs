use std::any;

use crate::database::HasRawValue;
use crate::{decode, Database, Error, RawValue, Result, TypeDecode, TypeInfo};

/// A single row from a result set generated from the database.
pub trait Row: 'static + Send + Sync {
    type Database: Database;

    /// Returns `true` if the row contains only `NULL` values.
    fn is_null(&self) -> bool;

    /// Returns the number of columns in the row.
    fn len(&self) -> usize;

    /// Returns `true` if there are no columns in the row.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the columns in the row.
    fn columns(&self) -> &[<Self::Database as Database>::Column];

    /// Returns the column at the index, if available.
    fn column<I: ColumnIndex<Self>>(&self, index: I) -> &<Self::Database as Database>::Column {
        self.try_column(index).unwrap()
    }

    /// Returns the column at the index, if available.
    fn try_column<I: ColumnIndex<Self>>(
        &self,
        index: I,
    ) -> Result<&<Self::Database as Database>::Column>;

    /// Returns the column name, given the index of the column.
    fn column_name(&self, index: usize) -> Option<&str>;

    /// Returns the column index, given the name of the column.
    fn column_index(&self, name: &str) -> Option<usize>;

    /// Returns the value for a column.
    ///
    /// # Panics
    ///
    /// Will panic for any errors documented in [`try_get`].
    ///
    fn get<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, Self::Database>,
    {
        self.try_get(index).unwrap()
    }

    /// Returns the _unchecked_ value for a column.
    ///
    /// # Panics
    ///
    /// Will panic for any errors documented in [`try_get_unchecked`].
    ///
    fn get_unchecked<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, Self::Database>,
    {
        self.try_get_unchecked(index).unwrap()
    }

    /// Returns the value for a column.
    ///
    /// # Errors
    ///
    /// -   Will return `Error::ColumnNotFound` or `Error::ColumnIndexOutOfBounds` if the
    ///     there is no column with the given name or index.
    ///
    /// -   Will return `Error::ColumnDecode` if there was an issue decoding
    ///     the value. Common reasons include:
    ///
    ///     -   The SQL value is `NULL` and `T` is not `Option<T>`
    ///
    ///     -   The SQL value cannot be represented as a `T`. Truncation or
    ///         loss of precision are considered errors.
    ///
    ///     -   The SQL type is not [`compatible`][Type::compatible] with
    ///         the Rust type.
    ///
    ///
    fn try_get<'r, T, I>(&'r self, index: I) -> Result<T>
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, Self::Database>,
    {
        let value = self.try_get_raw(&index)?;

        let res = if !value.is_null() && !T::compatible(value.type_info()) {
            Err(decode::Error::TypeNotCompatible {
                rust_type_name: any::type_name::<T>(),
                sql_type_name: value.type_info().name(),
            })
        } else {
            T::decode(value)
        };

        res.map_err(|err| Error::column_decode(self.column(&index), err))
    }

    /// Returns the _unchecked_ value for a column.
    ///
    /// In this case, _unchecked_ does not mean `unsafe`. Unlike [`try_get`],
    /// this method will not check that the source SQL value is compatible
    /// with the target Rust type. This may result in *weird* behavior (reading
    /// a `bool` from bytes of a `TEXT` column) but it is not `unsafe` and
    /// cannot cause undefined behavior.
    ///
    /// The type-compatible checks in SQLx will likely never be perfect. This
    /// method exists to work-around them in a controlled scenario.
    ///
    /// # Errors
    ///
    /// -   Will return `Error::ColumnNotFound` or `Error::ColumnIndexOutOfBounds` if the
    ///     there is no column with the given name or index.
    ///
    /// -   Will return `Error::ColumnDecode` if there was an issue decoding
    ///     the value. Common reasons include:
    ///
    ///     -   The SQL value is `NULL` and `T` is not `Option<T>`
    ///
    ///     -   The SQL value cannot be represented as a `T`. Truncation or
    ///         loss of precision are considered errors.
    ///
    fn try_get_unchecked<'r, T, I>(&'r self, index: I) -> Result<T>
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, Self::Database>,
    {
        let value = self.try_get_raw(&index)?;

        T::decode(value).map_err(|err| Error::column_decode(self.column(&index), err))
    }

    /// Returns the raw representation of the value for a column.
    ///
    /// # Panics
    ///
    /// Will panic for any errors documented in [`try_get_raw`].
    ///
    #[allow(clippy::needless_lifetimes)]
    fn get_raw<'r, I: ColumnIndex<Self>>(
        &'r self,
        index: I,
    ) -> <Self::Database as HasRawValue<'r>>::RawValue {
        self.try_get_raw(index).unwrap()
    }

    /// Returns the raw representation of the value for a column.
    ///
    /// # Errors
    ///
    /// -   Will return `Error::ColumnNotFound` or `Error::ColumnIndexOutOfBounds` if the
    ///     there is no column with the given name or index.
    ///
    #[allow(clippy::needless_lifetimes)]
    fn try_get_raw<'r, I: ColumnIndex<Self>>(
        &'r self,
        index: I,
    ) -> Result<<Self::Database as HasRawValue<'r>>::RawValue>;
}

/// A helper trait used for indexing into a [`Row`].
pub trait ColumnIndex<R: Row + ?Sized> {
    /// Returns the index of the column at this index, if present.
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> Result<usize>;
}

// access by index
impl<R: ?Sized + Row> ColumnIndex<R> for usize {
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> Result<usize> {
        if *self >= row.len() {
            return Err(Error::ColumnIndexOutOfBounds { len: row.len(), index: *self });
        }

        Ok(*self)
    }
}

// access by name
impl<R: ?Sized + Row> ColumnIndex<R> for &'_ str {
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> Result<usize> {
        row.column_index(self)
            .ok_or_else(|| Error::ColumnNotFound { name: self.to_string().into_boxed_str() })
    }
}

// access by reference
impl<R: ?Sized + Row, I: ColumnIndex<R>> ColumnIndex<R> for &'_ I {
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> Result<usize> {
        (*self).get(row)
    }
}
