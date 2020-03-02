//! Tests for the raw (unprepared) query API for Postgres.

use sqlx::{Cursor, Executor, Postgres, Row};
use sqlx_test::new;

/// Tests the edge case of executing a completely empty query string.
///
/// This gets flagged as an `EmptyQueryResponse` in Postgres. We currently
/// catch this and just return no rows.
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_empty_query() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let affected = conn.execute("").await?;

    assert_eq!(affected, 0);

    Ok(())
}

/// Test a simple select expression. This should return the row.
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_select_expression() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let mut cursor = conn.fetch("SELECT 5");
    let row = cursor.next().await?.unwrap();

    assert!(5i32 == row.get::<i32, _>(0)?);

    Ok(())
}

/// Test that we can interleave reads and writes to the database
/// in one simple query. Using the `Cursor` API we should be
/// able to fetch from both queries in sequence.
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_multi_read_write() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let mut cursor = conn.fetch(
        "
CREATE TABLE IF NOT EXISTS _sqlx_test_postgres_5112 (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL
);

SELECT 'Hello World' as _1;

INSERT INTO _sqlx_test_postgres_5112 (text) VALUES ('this is a test');

SELECT id, text FROM _sqlx_test_postgres_5112;
    ",
    );

    let row = cursor.next().await?.unwrap();

    assert!("Hello World" == row.get::<&str, _>("_1")?);

    let row = cursor.next().await?.unwrap();

    let id: i64 = row.get("id")?;
    let text: &str = row.get("text")?;

    assert_eq!(1_i64, id);
    assert_eq!("this is a test", text);

    Ok(())
}
