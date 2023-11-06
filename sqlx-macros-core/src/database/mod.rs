use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::Executor;

#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub enum ParamChecking {
    Strong,
    Weak,
}

pub trait DatabaseExt: Database {
    const DATABASE_PATH: &'static str;
    const ROW_PATH: &'static str;

    const PARAM_CHECKING: ParamChecking;

    fn db_path() -> syn::Path {
        syn::parse_str(Self::DATABASE_PATH).unwrap()
    }

    fn row_path() -> syn::Path {
        syn::parse_str(Self::ROW_PATH).unwrap()
    }

    fn param_type_for_id(id: &Self::TypeInfo) -> Option<&'static str>;

    fn return_type_for_id(id: &Self::TypeInfo) -> Option<&'static str>;

    fn get_feature_gate(info: &Self::TypeInfo) -> Option<&'static str>;

    fn describe_blocking(query: &str, database_url: &str) -> sqlx_core::Result<Describe<Self>>;
}

#[allow(dead_code)]
pub struct CachingDescribeBlocking<DB: DatabaseExt> {
    connections: Lazy<Mutex<HashMap<String, DB::Connection>>>,
}

#[allow(dead_code)]
impl<DB: DatabaseExt> CachingDescribeBlocking<DB> {
    pub const fn new() -> Self {
        CachingDescribeBlocking {
            connections: Lazy::new(|| Mutex::new(HashMap::new())),
        }
    }

    pub fn describe(&self, query: &str, database_url: &str) -> sqlx_core::Result<Describe<DB>>
    where
        for<'a> &'a mut DB::Connection: Executor<'a, Database = DB>,
    {
        crate::block_on(async {
            let mut cache = self
                .connections
                .lock()
                .expect("previous panic in describe call");

            let conn = match cache.entry(database_url.to_string()) {
                hash_map::Entry::Occupied(hit) => hit.into_mut(),
                hash_map::Entry::Vacant(miss) => {
                    miss.insert(DB::Connection::connect(&database_url).await?)
                }
            };

            conn.describe(query).await
        })
    }
}

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
macro_rules! impl_database_ext {
    (
        $database:path {
            $($(#[$meta:meta])? $ty:ty $(| $input:ty)?),*$(,)?
        },
        ParamChecking::$param_checking:ident,
        feature-types: $ty_info:ident => $get_gate:expr,
        row: $row:path,
        $(describe-blocking: $describe:path,)?
    ) => {
        impl $crate::database::DatabaseExt for $database {
            const DATABASE_PATH: &'static str = stringify!($database);
            const ROW_PATH: &'static str = stringify!($row);
            const PARAM_CHECKING: $crate::database::ParamChecking = $crate::database::ParamChecking::$param_checking;

            fn param_type_for_id(info: &Self::TypeInfo) -> Option<&'static str> {
                match () {
                    $(
                        $(#[$meta])?
                        _ if <$ty as sqlx_core::types::Type<$database>>::type_info() == *info => Some(input_ty!($ty $(, $input)?)),
                    )*
                    $(
                        $(#[$meta])?
                        _ if <$ty as sqlx_core::types::Type<$database>>::compatible(info) => Some(input_ty!($ty $(, $input)?)),
                    )*
                    _ => None
                }
            }

            fn return_type_for_id(info: &Self::TypeInfo) -> Option<&'static str> {
                match () {
                    $(
                        $(#[$meta])?
                        _ if <$ty as sqlx_core::types::Type<$database>>::type_info() == *info => return Some(stringify!($ty)),
                    )*
                    $(
                        $(#[$meta])?
                        _ if <$ty as sqlx_core::types::Type<$database>>::compatible(info) => return Some(stringify!($ty)),
                    )*
                    _ => None
                }
            }

            fn get_feature_gate($ty_info: &Self::TypeInfo) -> Option<&'static str> {
                $get_gate
            }

            impl_describe_blocking!($database, $($describe)?);
        }
    }
}

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
macro_rules! impl_describe_blocking {
    ($database:path $(,)?) => {
        fn describe_blocking(
            query: &str,
            database_url: &str,
        ) -> sqlx_core::Result<sqlx_core::describe::Describe<Self>> {
            use $crate::database::CachingDescribeBlocking;

            // This can't be a provided method because the `static` can't reference `Self`.
            static CACHE: CachingDescribeBlocking<$database> = CachingDescribeBlocking::new();

            CACHE.describe(query, database_url)
        }
    };
    ($database:path, $describe:path) => {
        fn describe_blocking(
            query: &str,
            database_url: &str,
        ) -> sqlx_core::Result<sqlx_core::describe::Describe<Self>> {
            $describe(query, database_url)
        }
    };
}

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
macro_rules! input_ty {
    ($ty:ty, $input:ty) => {
        stringify!($input)
    };
    ($ty:ty) => {
        stringify!($ty)
    };
}

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "sqlite")]
mod sqlite;

mod fake_sqlx {
    pub use sqlx_core::*;

    #[cfg(feature = "mysql")]
    pub use sqlx_mysql as mysql;

    #[cfg(feature = "postgres")]
    pub use sqlx_postgres as postgres;

    #[cfg(feature = "sqlite")]
    pub use sqlx_sqlite as sqlite;
}
