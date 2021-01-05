use crate::types::Type;

#[cfg(feature = "postgres")]
use crate::postgres::Postgres;

#[cfg(feature = "mysql")]
use crate::mysql::MySql;

#[cfg(feature = "mssql")]
use crate::mssql::Mssql;

#[cfg(feature = "sqlite")]
use crate::sqlite::Sqlite;

// Type is required by the bounds of the [`Row`] and [`Arguments`] trait but its been overridden in
// AnyRow and AnyArguments to not use this implementation; but instead, delegate to the
// database-specific implementation.
//
// The other use of this trait is for compile-time verification which is not feasible to support
// for the [`Any`] driver.
macro_rules! impl_any_type {
    ($ty:ty) => {
        impl crate::types::Type<crate::any::Any> for $ty
        where
            $ty: crate::any::AnyType,
        {
            fn type_info() -> crate::any::AnyTypeInfo {
                // FIXME: nicer panic explaining why this isn't possible
                unimplemented!()
            }

            fn compatible(ty: &crate::any::AnyTypeInfo) -> bool {
                match &ty.0 {
                    #[cfg(feature = "postgres")]
                    crate::any::type_info::AnyTypeInfoKind::Postgres(ty) => {
                        <$ty as crate::types::Type<crate::postgres::Postgres>>::compatible(&ty)
                    }

                    #[cfg(feature = "mysql")]
                    crate::any::type_info::AnyTypeInfoKind::MySql(ty) => {
                        <$ty as crate::types::Type<crate::mysql::MySql>>::compatible(&ty)
                    }

                    #[cfg(feature = "sqlite")]
                    crate::any::type_info::AnyTypeInfoKind::Sqlite(ty) => {
                        <$ty as crate::types::Type<crate::sqlite::Sqlite>>::compatible(&ty)
                    }

                    #[cfg(feature = "mssql")]
                    crate::any::type_info::AnyTypeInfoKind::Mssql(ty) => {
                        <$ty as crate::types::Type<crate::mssql::Mssql>>::compatible(&ty)
                    }
                }
            }
        }
    };
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
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<MySql> + Type<Mssql> + Type<Sqlite> {}

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
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<MySql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
pub trait AnyType: Type<Postgres> + Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
pub trait AnyType: Type<Postgres> + Type<MySql> + Type<Mssql> {}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<MySql> + Type<Mssql> {}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
pub trait AnyType: Type<Sqlite> + Type<MySql> + Type<Mssql> {}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
impl<T: ?Sized> AnyType for T where T: Type<Sqlite> + Type<MySql> + Type<Mssql> {}

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
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
pub trait AnyType: Type<Postgres> + Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
pub trait AnyType: Type<Postgres> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
impl<T: ?Sized> AnyType for T where T: Type<Postgres> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
pub trait AnyType: Type<Mssql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
impl<T: ?Sized> AnyType for T where T: Type<Mssql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
pub trait AnyType: Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
impl<T: ?Sized> AnyType for T where T: Type<Mssql> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
pub trait AnyType: Type<MySql> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
impl<T: ?Sized> AnyType for T where T: Type<MySql> + Type<Sqlite> {}

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
impl<T: ?Sized> AnyType for T where T: Type<Postgres> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
pub trait AnyType: Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
impl<T: ?Sized> AnyType for T where T: Type<MySql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
pub trait AnyType: Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
impl<T: ?Sized> AnyType for T where T: Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
pub trait AnyType: Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
impl<T: ?Sized> AnyType for T where T: Type<Sqlite> {}
