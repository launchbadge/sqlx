use futures::TryStreamExt;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{postgres::PgConnection, Connect, Executor, Row};
use std::time::Duration;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = sqlx::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO users (id) VALUES ($1)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(cnt, 1);
    }

    let sum: i32 = sqlx::query("SELECT id FROM users")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

// https://github.com/launchbadge/sqlx/issues/104
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_can_return_interleaved_nulls_issue_104() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let tuple =
        sqlx::query("SELECT NULL::INT, 10::INT, NULL, 20::INT, NULL, 40::INT, NULL, 80::INT")
            .try_map(|row: PgRow| {
                Ok((
                    row.get::<Option<i32>, _>(0),
                    row.get::<Option<i32>, _>(1),
                    row.get::<Option<i32>, _>(2),
                    row.get::<Option<i32>, _>(3),
                    row.get::<Option<i32>, _>(4),
                    row.get::<Option<i32>, _>(5),
                    row.get::<Option<i32>, _>(6),
                    row.get::<Option<i32>, _>(7),
                ))
            })
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(tuple.0, None);
    assert_eq!(tuple.1, Some(10));
    assert_eq!(tuple.2, None);
    assert_eq!(tuple.3, Some(20));
    assert_eq!(tuple.4, None);
    assert_eq!(tuple.5, Some(40));
    assert_eq!(tuple.6, None);
    assert_eq!(tuple.7, Some(80));

    Ok(())
}

// run with `cargo test --features postgres -- --ignored --nocapture pool_smoke_test`
#[ignore]
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn pool_smoke_test() -> anyhow::Result<()> {
    use sqlx_core::runtime::{sleep, spawn, timeout};

    eprintln!("starting pool");

    let pool = PgPool::builder()
        .connect_timeout(Duration::from_secs(5))
        .min_size(5)
        .max_size(10)
        .build(&dotenv::var("DATABASE_URL")?)
        .await?;

    // spin up more tasks than connections available, and ensure we don't deadlock
    for i in 0..20 {
        let pool = pool.clone();
        spawn(async move {
            loop {
                if let Err(e) = sqlx::query("select 1 + 1").execute(&pool).await {
                    eprintln!("pool task {} dying due to {}", i, e);
                    break;
                }
            }
        });
    }

    for _ in 0..5 {
        let pool = pool.clone();
        spawn(async move {
            while !pool.is_closed() {
                // drop acquire() futures in a hot loop
                // https://github.com/launchbadge/sqlx/issues/83
                drop(pool.acquire());
            }
        });
    }

    eprintln!("sleeping for 30 seconds");

    sleep(Duration::from_secs(30)).await;

    assert_eq!(pool.size(), 10);

    eprintln!("closing pool");

    timeout(Duration::from_secs(30), pool.close()).await?;

    eprintln!("pool closed successfully");

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_describe() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let _ = conn
        .execute(
            r#"
        CREATE TEMP TABLE describe_test (
            id SERIAL primary key,
            name text not null,
            hash bytea
        )
    "#,
        )
        .await?;

    let describe = conn
        .describe("select nt.*, false from describe_test nt")
        .await?;

    assert_eq!(describe.result_columns[0].non_null, Some(true));
    assert_eq!(describe.result_columns[0].type_info.type_name(), "INT4");
    assert_eq!(describe.result_columns[1].non_null, Some(true));
    assert_eq!(describe.result_columns[1].type_info.type_name(), "TEXT");
    assert_eq!(describe.result_columns[2].non_null, Some(false));
    assert_eq!(describe.result_columns[2].type_info.type_name(), "BYTEA");
    assert_eq!(describe.result_columns[3].non_null, None);
    assert_eq!(describe.result_columns[3].type_info.type_name(), "BOOL");

    Ok(())
}

async fn connect() -> anyhow::Result<PgConnection> {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();

    Ok(PgConnection::connect(dotenv::var("DATABASE_URL")?).await?)
}
