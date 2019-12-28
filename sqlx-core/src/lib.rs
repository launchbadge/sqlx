#![recursion_limit = "256"]
#![deny(unsafe_code)]

#[macro_use]
pub mod error;

#[cfg(any(feature = "mysql", feature = "postgres"))]
#[macro_use]
mod io;

#[cfg(any(feature = "mysql", feature = "postgres"))]
mod cache;

mod connection;
mod database;
mod executor;
mod query;
mod url;

pub mod arguments;
pub mod decode;
pub mod describe;
pub mod encode;
pub mod pool;
pub mod types;

#[macro_use]
pub mod row;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

pub use database::Database;

#[doc(inline)]
pub use error::{Error, Result};

pub use connection::Connection;
pub use executor::Executor;
pub use query::{query, Query};

#[doc(inline)]
pub use pool::Pool;

#[doc(inline)]
pub use row::{FromRow, Row};

#[cfg(feature = "mysql")]
#[doc(inline)]
pub use mysql::MySql;

#[cfg(feature = "postgres")]
#[doc(inline)]
pub use postgres::Postgres;

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
