use sqlx::{Connect, Database};

fn setup_if_needed() {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();
}

// Make a new connection
// Ensure [dotenv] and [env_logger] have been setup
pub async fn new<DB>() -> anyhow::Result<DB::Connection>
where
    DB: Database,
{
    setup_if_needed();

    Ok(DB::Connection::connect(dotenv::var("DATABASE_URL")?).await?)
}

// Test type encoding and decoding
#[macro_export]
macro_rules! test_type {
    ($name:ident($db:ident, $ty:ty, $($text:literal == $value:expr),+)) => {
        $crate::test_prepared_type!($name($db, $ty, $($text == $value),+));
        $crate::test_unprepared_type!($name($db, $ty, $($text == $value),+));
    }
}

// Test type decoding for the simple (unprepared) query API
#[macro_export]
macro_rules! test_unprepared_type {
    ($name:ident($db:ident, $ty:ty, $($text:literal == $value:expr),+)) => {
        paste::item! {
            #[cfg_attr(feature = "runtime-async-std", async_std::test)]
            #[cfg_attr(feature = "runtime-tokio", tokio::test)]
            async fn [< test_unprepared_type_ $name >] () -> anyhow::Result<()> {
                use sqlx::prelude::*;

                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!("SELECT {} as _1", $text);
                    let mut cursor = conn.fetch(&*query);
                    let row = cursor.next().await?.unwrap();
                    let rec = row.try_get::<$ty, _>("_1")?;

                    assert!($value == rec);
                )+

                Ok(())
            }
        }
    }
}

// Test type encoding and decoding for the prepared query API
#[macro_export]
macro_rules! test_prepared_type {
    ($name:ident($db:ident, $ty:ty, $($text:literal == $value:expr),+)) => {
        paste::item! {
            #[cfg_attr(feature = "runtime-async-std", async_std::test)]
            #[cfg_attr(feature = "runtime-tokio", tokio::test)]
            async fn [< test_prepared_type_ $name >] () -> anyhow::Result<()> {
                use sqlx::prelude::*;

                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!($crate::[< $db _query_for_test_prepared_type >]!(), $text);

                    let rec: (bool, $ty) = sqlx::query_as(&query)
                        .bind($value)
                        .bind($value)
                        .fetch_one(&mut conn)
                        .await?;

                    assert!(rec.0, "value returned from server: {:?}", rec.1);
                    assert!($value == rec.1);
                )+

                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! MySql_query_for_test_prepared_type {
    () => {
        "SELECT {} <=> ?, ? as _1"
    };
}

#[macro_export]
macro_rules! Postgres_query_for_test_prepared_type {
    () => {
        "SELECT {} is not distinct from $1, $2 as _1"
    };
}
