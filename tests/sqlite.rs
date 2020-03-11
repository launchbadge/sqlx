use futures::TryStreamExt;
use sqlx::{sqlite::SqliteQueryAs, Connection, Executor, Sqlite, SqlitePool, SqliteConnection, Connect};
use sqlx_test::new;
use std::time::Duration;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_connects() -> anyhow::Result<()> {
    Ok(new::<Sqlite>().await?.ping().await?)
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_fails_to_connect() -> anyhow::Result<()> {
    // empty connection string
    assert!(SqliteConnection::connect("").await.is_err());
    assert!(dbg!(SqliteConnection::connect("sqlite:///please_do_not_run_sqlx_tests_as_root").await).is_err());

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY)
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO users (id) VALUES (?)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(cnt, 1);
    }

    let sum: i32 = sqlx::query_as("SELECT id FROM users")
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, (x,): (i32,)| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}
