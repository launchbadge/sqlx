use futures::{FutureExt, TryFutureExt};
use sqlx::any::AnyPoolOptions;
use sqlx::prelude::*;
use sqlx_test::new;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[sqlx_macros::test]
async fn pool_should_invoke_after_connect() -> anyhow::Result<()> {
    let counter = Arc::new(AtomicUsize::new(0));

    let pool = AnyPoolOptions::new()
        .after_connect({
            let counter = counter.clone();
            move |conn| {
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

    assert_eq!(counter.load(Ordering::SeqCst), 1);

    Ok(())
}
