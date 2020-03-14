//! Tests for the raw (unprepared) query API for Sqlite.

use sqlx::{Cursor, Executor, Row, Sqlite};
use sqlx_test::new;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_multi_cursor() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let query = format!("SELECT {} as _1", 10);
    let mut cursor1 = conn.fetch(&*query);
    let mut cursor2 = conn.fetch(&*query);
    // let row = cursor.next().await?.unwrap();

    // assert!(5i32 == row.try_get::<i32, _>(0)?);

    Ok(())
}
