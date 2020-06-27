use crate::any::value::AnyValueRefKind;
use crate::any::{Any, AnyValueRef};
use crate::database::HasValueRef;
use crate::error::Error;
use crate::row::{ColumnIndex, Row};

#[cfg(feature = "postgres")]
use crate::postgres::PgRow;

#[cfg(feature = "mysql")]
use crate::mysql::MySqlRow;

#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteRow;

#[cfg(feature = "mssql")]
use crate::mssql::MssqlRow;

pub struct AnyRow(pub(crate) AnyRowKind);

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

    fn len(&self) -> usize {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyRowKind::Postgres(row) => row.len(),

            #[cfg(feature = "mysql")]
            AnyRowKind::MySql(row) => row.len(),

            #[cfg(feature = "sqlite")]
            AnyRowKind::Sqlite(row) => row.len(),

            #[cfg(feature = "mssql")]
            AnyRowKind::Mssql(row) => row.len(),
        }
    }

    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<<Self::Database as HasValueRef<'_>>::ValueRef, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;

        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyRowKind::Postgres(row) => row.try_get_raw(index).map(AnyValueRefKind::Postgres),

            #[cfg(feature = "mysql")]
            AnyRowKind::MySql(row) => row.try_get_raw(index).map(AnyValueRefKind::MySql),

            #[cfg(feature = "sqlite")]
            AnyRowKind::Sqlite(row) => row.try_get_raw(index).map(AnyValueRefKind::Sqlite),

            #[cfg(feature = "mssql")]
            AnyRowKind::Mssql(row) => row.try_get_raw(index).map(AnyValueRefKind::Mssql),
        }
        .map(AnyValueRef)
    }
}
