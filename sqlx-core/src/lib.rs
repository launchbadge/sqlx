#![recursion_limit="256"]
#![allow(unused_imports)]

#[macro_use]
mod macros;

#[macro_use]
pub mod error;

#[cfg(any(feature = "postgres", feature = "mariadb"))]
#[macro_use]
mod io;

mod backend;
pub mod decode;

#[cfg(any(feature = "postgres", feature = "mariadb"))]
mod url;

#[macro_use]
mod row;

mod connection;
mod executor;
mod pool;

#[macro_use]
pub mod params;

pub mod encode;
mod query;
pub mod types;

mod describe;

#[doc(inline)]
pub use self::{
    backend::Backend,
    connection::Connection,
    decode::Decode,
    encode::Encode,
    error::{Error, Result},
    executor::Executor,
    pool::Pool,
    query::{query, Query},
    row::{FromRow, Row},
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
