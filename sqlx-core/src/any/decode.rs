use crate::any::value::AnyValueRefKind;
use crate::any::{Any, AnyValueRef};
use crate::decode::Decode;
use crate::error::BoxDynError;
use crate::types::Type;

#[cfg(feature = "postgres")]
use crate::postgres::Postgres;

#[cfg(feature = "mysql")]
use crate::mysql::MySql;

#[cfg(feature = "mssql")]
use crate::mssql::Mssql;

#[cfg(feature = "sqlite")]
use crate::sqlite::Sqlite;

// Implements Decode for any T where T supports Decode for any database that has support currently
// compiled into SQLx
impl<'r, T> Decode<'r, Any> for T
where
    T: AnyDecode<'r>,
{
    fn decode(value: AnyValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.0 {
            #[cfg(feature = "mysql")]
            AnyValueRefKind::MySql(value) => <T as Decode<'r, crate::mysql::MySql>>::decode(value),

            #[cfg(feature = "sqlite")]
            AnyValueRefKind::Sqlite(value) => {
                <T as Decode<'r, crate::sqlite::Sqlite>>::decode(value)
            }

            #[cfg(feature = "mssql")]
            AnyValueRefKind::Mssql(value) => <T as Decode<'r, crate::mssql::Mssql>>::decode(value),

            #[cfg(feature = "postgres")]
            AnyValueRefKind::Postgres(value) => {
                <T as Decode<'r, crate::postgres::Postgres>>::decode(value)
            }
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
pub trait AnyDecode<'r>:
    Decode<'r, Postgres>
    + Type<Postgres>
    + Decode<'r, MySql>
    + Type<MySql>
    + Decode<'r, Mssql>
    + Type<Mssql>
    + Decode<'r, Sqlite>
    + Type<Sqlite>
{
}

#[cfg(all(
    feature = "postgres",
    feature = "mysql",
    feature = "mssql",
    feature = "sqlite"
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres>
        + Type<Postgres>
        + Decode<'r, MySql>
        + Type<MySql>
        + Decode<'r, Mssql>
        + Type<Mssql>
        + Decode<'r, Sqlite>
        + Type<Sqlite>
{
}

// only 3 (4)

#[cfg(all(
    not(feature = "mssql"),
    all(feature = "postgres", feature = "mysql", feature = "sqlite")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Postgres>
    + Type<Postgres>
    + Decode<'r, MySql>
    + Type<MySql>
    + Decode<'r, Sqlite>
    + Type<Sqlite>
{
}

#[cfg(all(
    not(feature = "mssql"),
    all(feature = "postgres", feature = "mysql", feature = "sqlite")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres>
        + Type<Postgres>
        + Decode<'r, MySql>
        + Type<MySql>
        + Decode<'r, Sqlite>
        + Type<Sqlite>
{
}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Postgres>
    + Type<Postgres>
    + Decode<'r, Mssql>
    + Type<Mssql>
    + Decode<'r, Sqlite>
    + Type<Sqlite>
{
}

#[cfg(all(
    not(feature = "mysql"),
    all(feature = "postgres", feature = "mssql", feature = "sqlite")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres>
        + Type<Postgres>
        + Decode<'r, Mssql>
        + Type<Mssql>
        + Decode<'r, Sqlite>
        + Type<Sqlite>
{
}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Postgres>
    + Type<Postgres>
    + Decode<'r, MySql>
    + Type<MySql>
    + Decode<'r, Mssql>
    + Type<Mssql>
{
}

#[cfg(all(
    not(feature = "sqlite"),
    all(feature = "postgres", feature = "mysql", feature = "mssql")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres>
        + Type<Postgres>
        + Decode<'r, MySql>
        + Type<MySql>
        + Decode<'r, Mssql>
        + Type<Mssql>
{
}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Sqlite>
    + Type<Sqlite>
    + Decode<'r, MySql>
    + Type<MySql>
    + Decode<'r, Mssql>
    + Type<Mssql>
{
}

#[cfg(all(
    not(feature = "postgres"),
    all(feature = "sqlite", feature = "mysql", feature = "mssql")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Sqlite>
        + Type<Sqlite>
        + Decode<'r, MySql>
        + Type<MySql>
        + Decode<'r, Mssql>
        + Type<Mssql>
{
}

// only 2 (6)

#[cfg(all(
    not(any(feature = "mssql", feature = "sqlite")),
    all(feature = "postgres", feature = "mysql")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Postgres> + Type<Postgres> + Decode<'r, MySql> + Type<MySql>
{
}

#[cfg(all(
    not(any(feature = "mssql", feature = "sqlite")),
    all(feature = "postgres", feature = "mysql")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres> + Type<Postgres> + Decode<'r, MySql> + Type<MySql>
{
}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Postgres> + Type<Postgres> + Decode<'r, Mssql> + Type<Mssql>
{
}

#[cfg(all(
    not(any(feature = "mysql", feature = "sqlite")),
    all(feature = "postgres", feature = "mssql")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres> + Type<Postgres> + Decode<'r, Mssql> + Type<Mssql>
{
}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Postgres> + Type<Postgres> + Decode<'r, Sqlite> + Type<Sqlite>
{
}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql")),
    all(feature = "postgres", feature = "sqlite")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Postgres> + Type<Postgres> + Decode<'r, Sqlite> + Type<Sqlite>
{
}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
pub trait AnyDecode<'r>: Decode<'r, Mssql> + Type<Mssql> + Decode<'r, MySql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "sqlite")),
    all(feature = "mssql", feature = "mysql")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Mssql> + Type<Mssql> + Decode<'r, MySql> + Type<MySql>
{
}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
pub trait AnyDecode<'r>:
    Decode<'r, Mssql> + Type<Mssql> + Decode<'r, Sqlite> + Type<Sqlite>
{
}

#[cfg(all(
    not(any(feature = "postgres", feature = "mysql")),
    all(feature = "mssql", feature = "sqlite")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, Mssql> + Type<Mssql> + Decode<'r, Sqlite> + Type<Sqlite>
{
}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
pub trait AnyDecode<'r>:
    Decode<'r, MySql> + Type<MySql> + Decode<'r, Sqlite> + Type<Sqlite>
{
}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql")),
    all(feature = "mysql", feature = "sqlite")
))]
impl<'r, T> AnyDecode<'r> for T where
    T: Decode<'r, MySql> + Type<MySql> + Decode<'r, Sqlite> + Type<Sqlite>
{
}

// only 1 (4)

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "sqlite")),
    feature = "postgres"
))]
pub trait AnyDecode<'r>: Decode<'r, Postgres> + Type<Postgres> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "sqlite")),
    feature = "postgres"
))]
impl<'r, T> AnyDecode<'r> for T where T: Decode<'r, Postgres> + Type<Postgres> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
pub trait AnyDecode<'r>: Decode<'r, MySql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "postgres", feature = "mssql", feature = "sqlite")),
    feature = "mysql"
))]
impl<'r, T> AnyDecode<'r> for T where T: Decode<'r, MySql> + Type<MySql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
pub trait AnyDecode<'r>: Decode<'r, Mssql> + Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "postgres", feature = "sqlite")),
    feature = "mssql"
))]
impl<'r, T> AnyDecode<'r> for T where T: Decode<'r, Mssql> + Type<Mssql> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
pub trait AnyDecode<'r>: Decode<'r, Sqlite> + Type<Sqlite> {}

#[cfg(all(
    not(any(feature = "mysql", feature = "mssql", feature = "postgres")),
    feature = "sqlite"
))]
impl<'r, T> AnyDecode<'r> for T where T: Decode<'r, Sqlite> + Type<Sqlite> {}
