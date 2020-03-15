//! Tests for the raw (unprepared) query API for Sqlite.

use sqlx::{Cursor, Executor, Row, Sqlite};
use sqlx_test::new;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_select_expression() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let mut cursor = conn.fetch("SELECT 5");
    let row = cursor.next().await?.unwrap();

    assert!(5i32 == row.try_get::<i32, _>(0)?);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_multi_read_write() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let mut cursor = conn.fetch(
        "
CREATE TABLE IF NOT EXISTS _sqlx_test (
    id INT PRIMARY KEY,
    text TEXT NOT NULL
);

SELECT 'Hello World' as _1;

INSERT INTO _sqlx_test (text) VALUES ('this is a test');

SELECT id, text FROM _sqlx_test;
    ",
    );

    let row = cursor.next().await?.unwrap();

    assert!("Hello World" == row.try_get::<&str, _>("_1")?);

    let row = cursor.next().await?.unwrap();

    let id: i64 = row.try_get("id")?;
    let text: &str = row.try_get("text")?;

    assert_eq!(0, id);
    assert_eq!("this is a test", text);

    Ok(())
}
