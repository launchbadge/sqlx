use sqlx_core::config;
use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::Executor;
use sqlx_core::sql_str::AssertSqlSafe;
use sqlx_core::sql_str::SqlSafeStr;
use sqlx_core::type_checking::TypeChecking;
use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[cfg(any(feature = "postgres", feature = "mysql", feature = "_sqlite"))]
mod impls;

pub trait DatabaseExt: Database + TypeChecking {
    const DATABASE_PATH: &'static str;
    const ROW_PATH: &'static str;

    fn db_path() -> syn::Path {
        syn::parse_str(Self::DATABASE_PATH).unwrap()
    }

    fn row_path() -> syn::Path {
        syn::parse_str(Self::ROW_PATH).unwrap()
    }

    fn describe_blocking(
        query: &str,
        database_url: &str,
        driver_config: &config::drivers::Config,
    ) -> sqlx_core::Result<Describe<Self>>;
}

#[allow(dead_code)]
pub struct CachingDescribeBlocking<DB: DatabaseExt> {
    connections: LazyLock<Mutex<HashMap<String, DB::Connection>>>,
}

#[allow(dead_code)]
impl<DB: DatabaseExt> CachingDescribeBlocking<DB> {
    #[allow(clippy::new_without_default, reason = "internal API")]
    pub const fn new() -> Self {
        CachingDescribeBlocking {
            connections: LazyLock::new(|| Mutex::new(HashMap::new())),
        }
    }

    pub fn describe(
        &self,
        query: &str,
        database_url: &str,
        _driver_config: &config::drivers::Config,
    ) -> sqlx_core::Result<Describe<DB>>
    where
        for<'a> &'a mut DB::Connection: Executor<'a, Database = DB>,
    {
        let mut cache = self
            .connections
            .lock()
            .expect("previous panic in describe call");

        crate::block_on(async {
            let conn = match cache.entry(database_url.to_string()) {
                hash_map::Entry::Occupied(hit) => hit.into_mut(),
                hash_map::Entry::Vacant(miss) => {
                    let conn = miss.insert(DB::Connection::connect(database_url).await?);

                    #[cfg(feature = "postgres")]
                    if DB::NAME == sqlx_postgres::Postgres::NAME {
                        conn.execute(
                            "
                            DO $$
                            BEGIN
                                IF EXISTS (
                                    SELECT 1
                                    FROM pg_settings
                                    WHERE name = 'plan_cache_mode'
                                ) THEN
                                    SET SESSION plan_cache_mode = 'force_generic_plan';
                                END IF;
                            END $$;
                        ",
                        )
                        .await?;
                    }
                    conn
                }
            };

            conn.describe(AssertSqlSafe(query.to_string()).into_sql_str())
                .await
        })
    }
}
