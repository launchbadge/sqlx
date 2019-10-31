#[macro_use]
mod macros;

#[cfg(any(feature = "postgres", feature = "mariadb"))]
#[macro_use]
mod io;

mod backend;
pub mod deserialize;

#[cfg(any(feature = "postgres", feature = "mariadb"))]
mod url;

#[macro_use]
mod row;

mod connection;
pub mod error;
mod executor;
mod pool;

#[doc(hidden)]
pub mod checked_sql;

#[macro_use]
pub mod query;

pub mod serialize;
mod sql;
pub mod types;

#[doc(inline)]
pub use self::{
    backend::Backend,
    connection::Connection,
    deserialize::FromSql,
    error::{Error, Result},
    executor::Executor,
    pool::Pool,
    row::{FromSqlRow, Row},
    serialize::ToSql,
    sql::{query, SqlQuery},
    types::HasSqlType,
};

#[cfg(feature = "mariadb")]
pub mod mariadb;

#[cfg(feature = "mariadb")]
#[doc(inline)]
pub use mariadb::MariaDb;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
#[doc(inline)]
pub use self::postgres::Postgres;
