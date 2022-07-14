use crate::any::error::mismatched_types;
use crate::any::{Any, AnyColumn, AnyColumnIndex};
use crate::column::ColumnIndex;
use crate::database::HasValueRef;
use crate::decode::Decode;
use crate::error::Error;
use crate::row::Row;
use crate::type_info::TypeInfo;
use crate::types::Type;
use crate::value::ValueRef;

#[cfg(feature = "postgres")]
use crate::postgres::PgRow;

#[cfg(feature = "mysql")]
use crate::mysql::MySqlRow;

#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteRow;

#[cfg(feature = "mssql")]
use crate::mssql::MssqlRow;

pub struct AnyRow {
    pub(crate) kind: AnyRowKind,
    pub(crate) columns: Vec<AnyColumn>,
}

impl crate::row::private_row::Sealed for AnyRow {}

pub(crate) enum AnyRowKind {
    #[cfg(feature = "postgres")]
    Postgres(PgRow),

    #[cfg(feature = "mysql")]
    MySql(MySqlRow),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteRow),

    #[cfg(feature = "mssql")]
    Mssql(MssqlRow),
}

impl Row for AnyRow {
    type Database = Any;

    fn columns(&self) -> &[AnyColumn] {
        &self.columns
    }

    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<<Self::Database as HasValueRef<'_>>::ValueRef, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;

        match &self.kind {
            #[cfg(feature = "postgres")]
            AnyRowKind::Postgres(row) => row.try_get_raw(index).map(Into::into),

            #[cfg(feature = "mysql")]
            AnyRowKind::MySql(row) => row.try_get_raw(index).map(Into::into),

            #[cfg(feature = "sqlite")]
            AnyRowKind::Sqlite(row) => row.try_get_raw(index).map(Into::into),

            #[cfg(feature = "mssql")]
            AnyRowKind::Mssql(row) => row.try_get_raw(index).map(Into::into),
        }
    }

    fn try_get<'r, T, I>(&'r self, index: I) -> Result<T, Error>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database> + Type<Self::Database>,
    {
        let value = self.try_get_raw(&index)?;
        let ty = value.type_info();

        if !value.is_null() && !ty.is_null() && !T::compatible(&ty) {
            Err(mismatched_types::<T>(&ty))
        } else {
            T::decode(value)
        }
        .map_err(|source| Error::ColumnDecode {
            index: format!("{:?}", index),
            source,
        })
    }
}

impl<'i> ColumnIndex<AnyRow> for &'i str
where
    &'i str: AnyColumnIndex,
{
    fn index(&self, row: &AnyRow) -> Result<usize, Error> {
        match &row.kind {
            #[cfg(feature = "postgres")]
            AnyRowKind::Postgres(row) => self.index(row),

            #[cfg(feature = "mysql")]
            AnyRowKind::MySql(row) => self.index(row),

            #[cfg(feature = "sqlite")]
            AnyRowKind::Sqlite(row) => self.index(row),

            #[cfg(feature = "mssql")]
            AnyRowKind::Mssql(row) => self.index(row),
        }
    }
}
