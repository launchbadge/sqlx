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

#[macro_use]
pub mod query;

pub mod serialize;
mod sql;
pub mod types;

mod prepared;

mod compiled;

#[doc(inline)]
pub use self::{
    backend::Backend,
    compiled::CompiledSql,
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

#[doc(hidden)]
pub use types::HasTypeMetadata;

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

#[cfg(feature = "uuid")]
pub use uuid::Uuid;

use std::marker::PhantomData;

// These types allow the `sqlx_macros::sql!()` macro to polymorphically compare a
// given parameter's type to an expected parameter type even if the former
// is behind a reference or in `Option`

#[doc(hidden)]
pub struct TyCons<T>(PhantomData<T>);

impl<T> TyCons<T> {
    pub fn new(_t: &T) -> TyCons<T> {
        TyCons(PhantomData)
    }
}

#[doc(hidden)]
pub trait TyConsExt: Sized {
    type Cons;
    fn ty_cons(self) -> Self::Cons {
        panic!("should not be run, only for type resolution")
    }
}

impl<T> TyCons<Option<&'_ T>> {
    pub fn ty_cons(self) -> T {
        panic!("should not be run, only for type resolution")
    }
}

impl<T> TyConsExt for TyCons<&'_ T> {
    type Cons = T;
}

impl<T> TyConsExt for TyCons<Option<T>> {
    type Cons = T;
}

impl<T> TyConsExt for &'_ TyCons<T> {
    type Cons = T;
}

#[test]
fn test_tycons_ext() {
    if false {
        let _: u64 = TyCons::new(&Some(5u64)).ty_cons();
        let _: u64 = TyCons::new(&Some(&5u64)).ty_cons();
        let _: u64 = TyCons::new(&&5u64).ty_cons();
        let _: u64 = TyCons::new(&5u64).ty_cons();
    }
}
