use crate::any::{Any, AnyTypeInfo};
use crate::column::Column;

#[cfg(feature = "postgres")]
use crate::postgres::PgColumn;

#[cfg(feature = "mysql")]
use crate::mysql::MySqlColumn;

#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteColumn;

#[cfg(feature = "mssql")]
use crate::mssql::MssqlColumn;

#[derive(Debug)]
pub struct AnyColumn {
    pub(crate) kind: AnyColumnKind,
    pub(crate) type_info: AnyTypeInfo,
}

impl crate::column::private_column::Sealed for AnyColumn {}

#[derive(Debug)]
pub(crate) enum AnyColumnKind {
    #[cfg(feature = "postgres")]
    Postgres(PgColumn),

    #[cfg(feature = "mysql")]
    MySql(MySqlColumn),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteColumn),

    #[cfg(feature = "mssql")]
    Mssql(MssqlColumn),
}

impl Column for AnyColumn {
    type Database = Any;

    fn ordinal(&self) -> usize {
        match &self.kind {
            #[cfg(feature = "postgres")]
            AnyColumnKind::Postgres(row) => row.ordinal(),

            #[cfg(feature = "mysql")]
            AnyColumnKind::MySql(row) => row.ordinal(),

            #[cfg(feature = "sqlite")]
            AnyColumnKind::Sqlite(row) => row.ordinal(),

            #[cfg(feature = "mssql")]
            AnyColumnKind::Mssql(row) => row.ordinal(),
        }
    }

    fn name(&self) -> &str {
        match &self.kind {
            #[cfg(feature = "postgres")]
            AnyColumnKind::Postgres(row) => row.name(),

            #[cfg(feature = "mysql")]
            AnyColumnKind::MySql(row) => row.name(),

            #[cfg(feature = "sqlite")]
            AnyColumnKind::Sqlite(row) => row.name(),

            #[cfg(feature = "mssql")]
            AnyColumnKind::Mssql(row) => row.name(),
        }
    }

    fn type_info(&self) -> &AnyTypeInfo {
        &self.type_info
    }
}
