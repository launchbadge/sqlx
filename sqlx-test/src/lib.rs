use sqlx::pool::PoolOptions;
use sqlx::{Connection, Database, Pool};
use std::env;

use anyhow::bail;
use futures::{future::poll_fn, pin_mut, task::Poll, Future};

pub fn setup_if_needed() {
    let _ = dotenv::dotenv();
    let _ = env_logger::builder().is_test(true).try_init();
}

// Make a new connection
// Ensure [dotenv] and [env_logger] have been setup
pub async fn new<DB>() -> anyhow::Result<DB::Connection>
where
    DB: Database,
{
    connect::<DB::Connection>().await
}

// prefer `new`
#[doc(hidden)]
pub async fn connect<C: Connection>() -> anyhow::Result<C> {
    setup_if_needed();
    Ok(C::connect(&env::var("DATABASE_URL")?).await?)
}

// Make a new pool
// Ensure [dotenv] and [env_logger] have been setup
pub async fn pool<DB>() -> anyhow::Result<Pool<DB>>
where
    DB: Database,
{
    setup_if_needed();

    let pool = PoolOptions::<DB>::new()
        .min_connections(0)
        .max_connections(5)
        .test_before_acquire(true)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    Ok(pool)
}

/// for N in 0 .. { create future, poll up to N times, drop future, test connection with ping() }
pub async fn test_cancellation<C, Tc, E>(conn: &mut C, mut test: Tc) -> anyhow::Result<()>
where
    C: Connection,
    Tc: for<'c> TestCancellation<'c, C, E>,
    anyhow::Error: From<E>,
{
    for max_polls in 0.. {
        let mut num_polls = 0;

        let mut last_await = {
            let fut = test.test_cancel(conn);
            pin_mut!(fut);

            let (res, mut last_await) = sqlx_rt::capture_last_await(poll_fn(move |cx| {
                if num_polls == max_polls {
                    return Poll::Ready(Ok(()));
                }

                num_polls += 1;

                fut.as_mut().poll(cx)
            }))
            .await;

            let _ = res?;

            if log::log_enabled!(log::Level::Trace) {
                if let Some(last_await) = last_await.as_mut() {
                    last_await.resolve();

                    log::trace!("last await backtrace: {:?}", last_await);
                }
            }

            last_await
        };

        if let Err(e) = conn.ping().await {
            if let Some(last_await) = last_await.as_mut() {
                last_await.resolve();
            }

            bail!(
                "test ended in broken connection after {} polls\nerror: {:?}\nlast await backtrace: {:?}",
                num_polls,
                e,
                last_await
            )
        }
    }

    Ok(())
}

// needed to work with a closure that returns a future which borrows its argument
pub trait TestCancellation<'c, C: Connection + 'c, E>
where
    anyhow::Error: From<E>,
{
    type Future: Future<Output = Result<(), E>> + 'c;

    fn test_cancel(&mut self, conn: &'c mut C) -> Self::Future;
}

impl<'c, C, F, Fut, E> TestCancellation<'c, C, E> for F
where
    C: Connection + 'c,
    F: FnMut(&'c mut C) -> Fut,
    Fut: Future<Output = Result<(), E>> + 'c,
    anyhow::Error: From<E>,
{
    type Future = Fut;

    fn test_cancel(&mut self, conn: &'c mut C) -> Self::Future {
        (self)(conn)
    }
}

// Test type encoding and decoding
#[macro_export]
macro_rules! test_type {
    ($name:ident<$ty:ty>($db:ident, $sql:literal, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_type!($name<$ty>($db, $sql, $($text == $value),+));
        $crate::test_unprepared_type!($name<$ty>($db, $($text == $value),+));
    };

    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            $crate::__test_prepared_type!($name<$ty>($db, $crate::[< $db _query_for_test_prepared_type >]!(), $($text == $value),+));
            $crate::test_unprepared_type!($name<$ty>($db, $($text == $value),+));
        }
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::test_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type decoding only
#[macro_export]
macro_rules! test_decode_type {
    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_decode_type!($name<$ty>($db, $($text == $value),+));
        $crate::test_unprepared_type!($name<$ty>($db, $($text == $value),+));
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::test_decode_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type encoding and decoding
#[macro_export]
macro_rules! test_prepared_type {
    ($name:ident<$ty:ty>($db:ident, $sql:literal, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_type!($name<$ty>($db, $sql, $($text == $value),+));
    };

    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            $crate::__test_prepared_type!($name<$ty>($db, $crate::[< $db _query_for_test_prepared_type >]!(), $($text == $value),+));
        }
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type decoding for the simple (unprepared) query API
#[macro_export]
macro_rules! test_unprepared_type {
    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            #[sqlx_macros::test]
            async fn [< test_unprepared_type_ $name >] () -> anyhow::Result<()> {
                use sqlx::prelude::*;
                use futures::TryStreamExt;

                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!("SELECT {}", $text);
                    let mut s = conn.fetch(&*query);
                    let row = s.try_next().await?.unwrap();
                    let rec = row.try_get::<$ty, _>(0)?;

                    assert!($value == rec);

                    drop(s);
                )+

                Ok(())
            }
        }
    }
}

// Test type decoding only for the prepared query API
#[macro_export]
macro_rules! __test_prepared_decode_type {
    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            #[sqlx_macros::test]
            async fn [< test_prepared_decode_type_ $name >] () -> anyhow::Result<()> {
                use sqlx::Row;

                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!("SELECT {}", $text);

                    let row = sqlx::query(&query)
                        .fetch_one(&mut conn)
                        .await?;

                    let rec: $ty = row.try_get(0)?;

                    assert!($value == rec);
                )+

                Ok(())
            }
        }
    };
}

// Test type encoding and decoding for the prepared query API
#[macro_export]
macro_rules! __test_prepared_type {
    ($name:ident<$ty:ty>($db:ident, $sql:expr, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            #[sqlx_macros::test]
            async fn [< test_prepared_type_ $name >] () -> anyhow::Result<()> {
                use sqlx::Row;

                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!($sql, $text);

                    let row = sqlx::query(&query)
                        .bind($value)
                        .bind($value)
                        .fetch_one(&mut conn)
                        .await?;

                    let matches: i32 = row.try_get(0)?;
                    let returned: $ty = row.try_get(1)?;
                    let round_trip: $ty = row.try_get(2)?;

                    assert!(matches != 0,
                            "[1] DB value mismatch; given value: {:?}\n\
                             as returned: {:?}\n\
                             round-trip: {:?}",
                            $value, returned, round_trip);

                    assert_eq!($value, returned,
                            "[2] DB value mismatch; given value: {:?}\n\
                                     as returned: {:?}\n\
                                     round-trip: {:?}",
                                    $value, returned, round_trip);

                    assert_eq!($value, round_trip,
                            "[3] DB value mismatch; given value: {:?}\n\
                                     as returned: {:?}\n\
                                     round-trip: {:?}",
                                    $value, returned, round_trip);
                )+

                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! MySql_query_for_test_prepared_type {
    () => {
        "SELECT {0} <=> ?, {0}, ?"
    };
}

#[macro_export]
macro_rules! Mssql_query_for_test_prepared_type {
    () => {
        "SELECT CASE WHEN {0} IS NULL AND @p1 IS NULL THEN 1 WHEN {0} = @p1 THEN 1 ELSE 0 END, {0}, @p2"
    };
}

#[macro_export]
macro_rules! Sqlite_query_for_test_prepared_type {
    () => {
        "SELECT {0} is ?, {0}, ?"
    };
}

#[macro_export]
macro_rules! Postgres_query_for_test_prepared_type {
    () => {
        "SELECT ({0} is not distinct from $1)::int4, {0}, $2"
    };
}
