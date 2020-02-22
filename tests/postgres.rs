use futures::TryStreamExt;
use sqlx::{
    postgres::PgConnection, Connect, Connection, Cursor, Database, Executor, Postgres, Row,
};
use sqlx_core::postgres::{PgPool, PgRow};
use std::time::Duration;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let row = sqlx::query("select 1 + 1").fetch_one(&mut conn).await?;

    assert_eq!(2i32, row.get::<i32, _>(0));

    conn.close().await?;

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

    let sum: i32 = sqlx::query::<Postgres>("SELECT id FROM users")
        .map(|row: PgRow| Ok(row.get::<i32, _>(0)))
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

    let row = sqlx::query("SELECT NULL::INT, 10::INT, NULL, 20::INT, NULL, 40::INT, NULL, 80::INT")
        .fetch_one(&mut conn)
        .await?;

    let _1: Option<i32> = row.get(0);
    let _2: Option<i32> = row.get(1);
    let _3: Option<i32> = row.get(2);
    let _4: Option<i32> = row.get(3);
    let _5: Option<i32> = row.get(4);
    let _6: Option<i32> = row.get(5);
    let _7: Option<i32> = row.get(6);
    let _8: Option<i32> = row.get(7);

    assert_eq!(_1, None);
    assert_eq!(_2, Some(10));
    assert_eq!(_3, None);
    assert_eq!(_4, Some(20));
    assert_eq!(_5, None);
    assert_eq!(_6, Some(40));
    assert_eq!(_7, None);
    assert_eq!(_8, Some(80));

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
                if let Err(e) = sqlx::query("select 1 + 1").fetch_one(&pool).await {
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

async fn connect() -> anyhow::Result<PgConnection> {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();
    Ok(PgConnection::connect(dotenv::var("DATABASE_URL")?).await?)
}
