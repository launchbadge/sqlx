use sqlx::mssql::{Mssql, MssqlIsolationLevel};
use sqlx::Row;
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_begins_with_read_uncommitted() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let mut tx = conn
        .begin_with_isolation(MssqlIsolationLevel::ReadUncommitted)
        .await?;

    let row = sqlx::query("SELECT 1 AS val").fetch_one(&mut *tx).await?;
    let val: i32 = row.get("val");
    assert_eq!(val, 1);

    tx.commit().await?;
    Ok(())
}

#[sqlx_macros::test]
async fn it_begins_with_snapshot() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    // Enable snapshot isolation on the database first
    sqlx::query("ALTER DATABASE CURRENT SET ALLOW_SNAPSHOT_ISOLATION ON")
        .execute(&mut conn)
        .await?;

    let mut tx = conn
        .begin_with_isolation(MssqlIsolationLevel::Snapshot)
        .await?;

    let row = sqlx::query("SELECT 1 AS val").fetch_one(&mut *tx).await?;
    let val: i32 = row.get("val");
    assert_eq!(val, 1);

    tx.commit().await?;
    Ok(())
}

#[sqlx_macros::test]
async fn it_begins_with_serializable() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let mut tx = conn
        .begin_with_isolation(MssqlIsolationLevel::Serializable)
        .await?;

    let row = sqlx::query("SELECT 1 AS val").fetch_one(&mut *tx).await?;
    let val: i32 = row.get("val");
    assert_eq!(val, 1);

    tx.commit().await?;
    Ok(())
}
