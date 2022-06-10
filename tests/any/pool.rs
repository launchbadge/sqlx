use sqlx::{any::AnyPoolOptions, pool::PoolMetricsObserver};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
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

    let recorder_state = Arc::new(Mutex::new(Vec::with_capacity(2)));
    let metrics = Arc::new(MetricRecorder {
        wait: Arc::clone(&recorder_state),
    });

    let pool = Arc::new(
        AnyPoolOptions::new()
            .max_connections(1)
            .metrics_observer(Arc::clone(&metrics))
            .connect(&dotenv::var("DATABASE_URL")?)
            .await?,
    );

    let conn_1 = pool.acquire().await?;

    // Grab a timestamp before conn_2 starts waiting, and compute the duration
    // once the task handle is joined to derive an upper bound on the pool wait
    // time.
    let started_at = std::time::Instant::now();

    // A signal to indicate the second acquisition task spawned below has been
    // scheduled and is executing.
    let (tx, spawned) = futures::channel::oneshot::channel();

    // This acquire blocks for conn_1 to be returned to the pool
    let handle = sqlx_rt::spawn({
        let pool = Arc::clone(&pool);
        async move {
            tx.send(()).expect("test not listening");
            let _conn_2 = pool.acquire().await?;
            Result::<(), anyhow::Error>::Ok(())
        }
    });

    // Wait for the second acquisition attempt to spawn and begin executing.
    spawned.await.expect("task panic");

    // Wait a known duration of time and then drop conn_1, unblocking conn_2.
    sqlx_rt::sleep(Duration::from_millis(DELAY_MS)).await;
    drop(conn_1);

    // Allow conn_2 to be acquired, and then immediately returning, joining the
    // task handle.
    let _ = handle.await.expect("acquire() task failed");

    // At this point, conn_2 would have been acquired and immediately dropped.
    //
    // Now conn_2 has definitely stopped waiting (as acquire() returned and the
    // task was joined), the upper bound on pool wait time can be derived.
    let upper_bound = started_at.elapsed();

    // Inspecting the wait times should show 2 permit acquisitions.
    let waits = recorder_state.lock().unwrap().clone();
    assert_eq!(waits.len(), 2);

    // We can derive a upper and lower bound for the permit acquisition duration
    // of conn_2, and use it to verify it is correctly recorded.
    //
    // The permit wait time MUST be at least, or equal to, DELAY_MS and no more
    // than upper_bound.
    let wait = waits[1];
    assert!(
        wait.as_millis() as u64 >= DELAY_MS,
        "expected at least {}, got {} when validating {:?}",
        DELAY_MS,
        wait.as_millis(),
        waits,
    );
    assert!(
        wait < upper_bound,
        "expected at most {:?}, got {:?} when validating {:?}",
        upper_bound,
        wait,
        waits
    );

    Ok(())
}

// Static asserts that various types are accepted.
#[sqlx_macros::test]
fn assert_metric_types() {
    let metrics = MetricRecorder {
        wait: Default::default(),
    };

    AnyPoolOptions::new()
        .metrics_observer(metrics.clone())
        .metrics_observer(Arc::new(metrics));
}

#[derive(Debug, Default, Clone)]
struct MetricRecorder {
    wait: Arc<Mutex<Vec<Duration>>>,
}

impl PoolMetricsObserver for MetricRecorder {
    fn permit_wait_time(&self, time: Duration) {
        self.wait.lock().unwrap().push(time)
    }
}
