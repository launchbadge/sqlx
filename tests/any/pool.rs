use sqlx::any::AnyPoolOptions;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

#[sqlx_macros::test]
async fn pool_should_invoke_after_connect() -> anyhow::Result<()> {
    let counter = Arc::new(AtomicUsize::new(0));

    let pool = AnyPoolOptions::new()
        .after_connect({
            let counter = counter.clone();
            move |_conn| {
                let counter = counter.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);

                    Ok(())
                })
            }
        })
        .connect(&dotenv::var("DATABASE_URL")?)
        .await?;

    let _ = pool.acquire().await?;
    let _ = pool.acquire().await?;
    let _ = pool.acquire().await?;
    let _ = pool.acquire().await?;

    // since connections are released asynchronously,
    // `.after_connect()` may be called more than once
    assert!(counter.load(Ordering::SeqCst) >= 1);

    Ok(())
}

// https://github.com/launchbadge/sqlx/issues/527
#[sqlx_macros::test]
async fn pool_should_be_returned_failed_transactions() -> anyhow::Result<()> {
    let pool = AnyPoolOptions::new()
        .max_connections(2)
        .connect_timeout(Duration::from_secs(3))
        .connect(&dotenv::var("DATABASE_URL")?)
        .await?;

    let query = "blah blah";

    let mut tx = pool.begin().await?;
    let res = sqlx::query(query).execute(&mut tx).await;
    assert!(res.is_err());
    drop(tx);

    let mut tx = pool.begin().await?;
    let res = sqlx::query(query).execute(&mut tx).await;
    assert!(res.is_err());
    drop(tx);

    let mut tx = pool.begin().await?;
    let res = sqlx::query(query).execute(&mut tx).await;
    assert!(res.is_err());
    drop(tx);

    Ok(())
}

#[sqlx_macros::test]
async fn pool_wait_duration_counter_increases() -> anyhow::Result<()> {
    const DELAY_MS: u64 = 10;

    let pool = Arc::new(
        AnyPoolOptions::new()
            .max_connections(1)
            .connect(&dotenv::var("DATABASE_URL")?)
            .await?,
    );

    let conn_1 = pool.acquire().await?;

    // This acquire blocks for conn_1 to be returned to the pool
    let handle = sqlx_rt::spawn({
        let pool = Arc::clone(&pool);
        async move {
            let _conn_2 = pool.acquire().await?;
            Result::<(), anyhow::Error>::Ok(())
        }
    });

    // Wait a known duration of time and then drop conn_1, unblocking conn_2.
    sqlx_rt::sleep(Duration::from_millis(DELAY_MS)).await;
    drop(conn_1);

    // Allow conn_2 to be acquired, and then immediately returning, joining the
    // task handle.
    let _ = handle.await.expect("acquire() task failed");

    // At this point, conn_2 would have been acquired and immediately dropped.
    //
    // The duration of time conn_2 was blocked should be recorded in the pool
    // wait metric.
    let wait = pool.pool_wait_duration();
    assert!(
        wait.as_millis() as u64 >= DELAY_MS,
        "expected at least {}, got {}",
        DELAY_MS,
        wait.as_millis()
    );

    Ok(())
}
