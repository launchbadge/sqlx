//! Tests for the raw (unprepared) query API for MySql.

use sqlx::{Cursor, Executor, MySql, Row};
use sqlx_test::new;

/// Test a simple select expression. This should return the row.
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_select_expression() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let mut cursor = conn.fetch("SELECT 5");
    let row = cursor.next().await?.unwrap();

    assert!(5i32 == row.try_get::<i32, _>(0)?);

    Ok(())
}

/// Test that we can interleave reads and writes to the database
/// in one simple query. Using the `Cursor` API we should be
/// able to fetch from both queries in sequence.
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_multi_read_write() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let mut cursor = conn.fetch(
        "
CREATE TEMPORARY TABLE messages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    text TEXT NOT NULL
);

SELECT 'Hello World' as _1;

INSERT INTO messages (text) VALUES ('this is a test');

SELECT id, text FROM messages;
        ",
    );

    let row = cursor.next().await?.unwrap();

    assert!("Hello World" == row.try_get::<&str, _>("_1")?);

    let row = cursor.next().await?.unwrap();

    let id: i64 = row.try_get("id")?;
    let text: &str = row.try_get("text")?;

    assert_eq!(1_i64, id);
    assert_eq!("this is a test", text);

    Ok(())
}
