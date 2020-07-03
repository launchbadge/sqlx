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

impl<'i> ColumnIndex<AnyRow> for &'i str
where
    &'i str: AnyColumnIndex,
{
    fn index(&self, row: &AnyRow) -> Result<usize, Error> {
        match &row.0 {
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

// FIXME: Find a nice way to auto-generate the below or petition Rust to add support for #[cfg]
//        to trait bounds

// all 4

#[cfg(all(
    feature = "postgres",
    feature = "mysql",
    feature = "mssql",
    feature = "sqlite"
))]
pub trait AnyColumnIndex:
    ColumnIndex<PgRow> + ColumnIndex<MySqlRow> + ColumnIndex<MssqlRow> + ColumnIndex<SqliteRow>
{
}

#[cfg(all(
    feature = "postgres",
    feature = "mysql",
    feature = "mssql",
    feature = "sqlite"
))]
impl<T: ?Sized> AnyColumnIndex for T where
    T: ColumnIndex<PgRow> + ColumnIndex<MySqlRow> + ColumnIndex<MssqlRow> + ColumnIndex<SqliteRow>
{
}

// only 3 (4)

#[cfg(all(
    not(feature = "mssql"),
    all(feature = "postgres", feature = "mysql", feature = "sqlite")
))]
pub trait AnyColumnIndex:
    ColumnIndex<PgRow> + ColumnIndex<MySqlRow> + ColumnIndex<SqliteRow>
{
}

#[cfg(all(
    not(feature = "mssql"),
    all(feature = "postgres", feature = "mysql", feature = "sqlite")
))]
impl<T: ?Sized> AnyColumnIndex for T where
    T: ColumnIndex<PgRow> + ColumnIndex<MySqlRow> + ColumnIndex<SqliteRow>
{
}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
pub trait AnyColumnIndex:
    ColumnIndex<PgRow> + ColumnIndex<MssqlRow> + ColumnIndex<SqliteRow>
{
}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
impl<T: ?Sized> AnyColumnIndex for T where
    T: ColumnIndex<PgRow> + ColumnIndex<MssqlRow> + ColumnIndex<SqliteRow>
{
}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
pub trait AnyColumnIndex:
    ColumnIndex<PgRow> + ColumnIndex<MySqlRow> + ColumnIndex<MssqlRow>
{
}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
impl<T: ?Sized> AnyColumnIndex for T where
    T: ColumnIndex<PgRow> + ColumnIndex<MySqlRow> + ColumnIndex<MssqlRow>
{
}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
pub trait AnyColumnIndex:
    ColumnIndex<SqliteRow> + ColumnIndex<MySqlRow> + ColumnIndex<MssqlRow>
{
}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
impl<T: ?Sized> AnyColumnIndex for T where
    T: ColumnIndex<SqliteRow> + ColumnIndex<MySqlRow> + ColumnIndex<MssqlRow>
{
}

// only 2 (6)

#[cfg(all(
    not(any(feature = "mssql", feature = "sqlite")),
    all(feature = "postgres", feature = "mysql")
))]
pub trait AnyColumnIndex: ColumnIndex<PgRow> + ColumnIndex<MySqlRow> {}

#[cfg(all(
    not(any(feature = "mssql", feature = "sqlite")),
    all(feature = "postgres", feature = "mysql")
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<PgRow> + ColumnIndex<MySqlRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
pub trait AnyColumnIndex: ColumnIndex<PgRow> + ColumnIndex<MssqlRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<PgRow> + ColumnIndex<MssqlRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
pub trait AnyColumnIndex: ColumnIndex<PgRow> + ColumnIndex<SqliteRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<PgRow> + ColumnIndex<SqliteRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
pub trait AnyColumnIndex: ColumnIndex<MssqlRow> + ColumnIndex<MySqlRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<MssqlRow> + ColumnIndex<MySqlRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
pub trait AnyColumnIndex: ColumnIndex<MssqlRow> + ColumnIndex<SqliteRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<MssqlRow> + ColumnIndex<SqliteRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
pub trait AnyColumnIndex: ColumnIndex<MySqlRow> + ColumnIndex<SqliteRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<MySqlRow> + ColumnIndex<SqliteRow> {}

// only 1 (4)

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "sqlite")),
    feature = "postgres"
))]
pub trait AnyColumnIndex: ColumnIndex<PgRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "sqlite")),
    feature = "postgres"
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<PgRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
pub trait AnyColumnIndex: ColumnIndex<MySqlRow> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<MySqlRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
pub trait AnyColumnIndex: ColumnIndex<MssqlRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<MssqlRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
pub trait AnyColumnIndex: ColumnIndex<SqliteRow> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
impl<T: ?Sized> AnyColumnIndex for T where T: ColumnIndex<SqliteRow> {}
