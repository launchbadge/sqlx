use crate::any::type_info::AnyTypeInfoKind;
use crate::any::{Any, AnyTypeInfo};
use crate::database::Database;
use crate::types::Type;

#[cfg(feature = "postgres")]
use crate::postgres::Postgres;

#[cfg(feature = "mysql")]
use crate::mysql::MySql;

#[cfg(feature = "mssql")]
use crate::mssql::Mssql;

#[cfg(feature = "sqlite")]
use crate::sqlite::Sqlite;

// Type is required by the bounds of the [Row] and [Arguments] trait but its been overridden in
// AnyRow and AnyArguments to not use this implementation; but instead, delegate to the
// database-specific implementation.
//
// The other use of this trait is for compile-time verification which is not feasible to support
// for the [Any] driver.
impl<T> Type<Any> for T
where
    T: AnyType,
{
    fn type_info() -> <Any as Database>::TypeInfo {
        // FIXME: nicer panic explaining why this isn't possible
        unimplemented!()
    }

    fn compatible(ty: &AnyTypeInfo) -> bool {
        match &ty.0 {
            #[cfg(feature = "postgres")]
            AnyTypeInfoKind::Postgres(ty) => <T as Type<Postgres>>::compatible(&ty),

            #[cfg(feature = "mysql")]
            AnyTypeInfoKind::MySql(ty) => <T as Type<MySql>>::compatible(&ty),

            #[cfg(feature = "sqlite")]
            AnyTypeInfoKind::Sqlite(ty) => <T as Type<Sqlite>>::compatible(&ty),

            #[cfg(feature = "mssql")]
            AnyTypeInfoKind::Mssql(ty) => <T as Type<Mssql>>::compatible(&ty),
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
pub trait AnyType: Type<Postgres> + Type<MySql> + Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    feature = "postgres",
    feature = "mysql",
    feature = "mssql",
    feature = "sqlite"
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<MySql> + Type<Mssql> + Type<Sqlite> {}

// only 3 (4)

#[cfg(all(
    not(feature = "mssql"),
    all(feature = "postgres", feature = "mysql", feature = "sqlite")
))]
pub trait AnyType: Type<Postgres> + Type<MySql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "mssql"),
    all(feature = "postgres", feature = "mysql", feature = "sqlite")
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<MySql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
pub trait AnyType: Type<Postgres> + Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
pub trait AnyType: Type<Postgres> + Type<MySql> + Type<Mssql> {}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<MySql> + Type<Mssql> {}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
pub trait AnyType: Type<Sqlite> + Type<MySql> + Type<Mssql> {}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
impl<T> AnyType for T where T: Type<Sqlite> + Type<MySql> + Type<Mssql> {}

// only 2 (6)

#[cfg(all(
    not(any(feature = "mssql", feature = "sqlite")),
    all(feature = "postgres", feature = "mysql")
))]
pub trait AnyType: Type<Postgres> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "mssql", feature = "sqlite")),
    all(feature = "postgres", feature = "mysql")
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
pub trait AnyType: Type<Postgres> + Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
pub trait AnyType: Type<Postgres> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
impl<T> AnyType for T where T: Type<Postgres> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
pub trait AnyType: Type<Mssql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
impl<T> AnyType for T where T: Type<Mssql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
pub trait AnyType: Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
impl<T> AnyType for T where T: Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
pub trait AnyType: Type<MySql> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
impl<T> AnyType for T where T: Type<MySql> + Type<Sqlite> {}

// only 1 (4)

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "sqlite")),
    feature = "postgres"
))]
pub trait AnyType: Type<Postgres> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "sqlite")),
    feature = "postgres"
))]
impl<T> AnyType for T where T: Type<Postgres> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
pub trait AnyType: Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
impl<T> AnyType for T where T: Type<MySql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
pub trait AnyType: Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
impl<T> AnyType for T where T: Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
pub trait AnyType: Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
impl<T> AnyType for T where T: Type<Sqlite> {}
