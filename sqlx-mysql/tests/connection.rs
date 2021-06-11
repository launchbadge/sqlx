use sqlx_core::{Connect, Connection, Executor, Tokio};
use sqlx_mysql::MySqlArguments;
use sqlx_mysql::MySqlConnection;
use sqlx_test::assert_cancellation_safe;
use std::env;

#[tokio::test]
async fn test_connect() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = MySqlConnection::<Tokio>::connect(&url).await?;

    conn.ping().await?;

    Ok(())
}

#[tokio::test]
async fn test_select_1() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = MySqlConnection::<Tokio>::connect(&url).await?;

    let row = conn.fetch_one("SELECT 1").await?;
    let col0: i32 = row.try_get(0)?;

    assert_eq!(col0, 1);

    Ok(())
}

#[tokio::test]
async fn test_ping_cancel() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = MySqlConnection::<Tokio>::connect(&url).await?;

    assert_cancellation_safe(&mut conn, |conn| conn.ping(), |conn| conn.ping()).await?;

    Ok(())
}

#[tokio::test]
async fn test_select_cancel() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = MySqlConnection::<Tokio>::connect(&url).await?;

    assert_cancellation_safe(
        &mut conn,
        |conn| {
            let mut args = MySqlArguments::new();
            args.add_unchecked(&1_i32);

            conn.fetch_one(("SELECT ?", args))
        },
        |conn| conn.ping(),
    )
    .await?;

    Ok(())
}
