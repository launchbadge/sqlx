use sqlx_core::{Connect, Connection, Executor, Tokio};
use sqlx_postgres::PgArguments;
use sqlx_postgres::PgConnection;
use sqlx_test::assert_cancellation_safe;
use std::env;

#[tokio::test]
async fn test_connect() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = PgConnection::<Tokio>::connect(&url).await?;

    conn.ping().await?;

    Ok(())
}

#[tokio::test]
async fn test_select_1() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = PgConnection::<Tokio>::connect(&url).await?;

    let row = conn.fetch_one("SELECT 1").await?;
    let col0: i32 = row.try_get(0)?;

    assert_eq!(col0, 1);

    Ok(())
}

#[tokio::test]
async fn test_generic_placeholders() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = PgConnection::<Tokio>::connect(&url).await?;

    let mut args = PgArguments::new();
    args.add(&1i32);

    let row = conn.fetch_one(("SELECT {}", args)).await?;
    let col0: i32 = row.try_get(0)?;

    let mut args = PgArguments::new();
    args.add(&[1i32, 2, 3, 4, 5, 6]);

    let row = conn
        .fetch_one((
            "SELECT val FROM generate_series(0, 9, 3) AS vals(val) WHERE val IN ({+})",
            args,
        ))
        .await?;
    let col0: i32 = row.try_get(0)?;

    assert_eq!(col0, 3);

    Ok(())
}
