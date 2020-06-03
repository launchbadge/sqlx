use sqlx::mssql::MsSql;
use sqlx::{Connection, Executor, Row};
use sqlx_core::mssql::MsSqlRow;
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<MsSql>().await?;

    conn.ping().await?;

    conn.close().await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_select_1() -> anyhow::Result<()> {
    let mut conn = new::<MsSql>().await?;

    let row: MsSqlRow = conn.fetch_one("SELECT 4").await?;
    let v: i32 = row.try_get(0)?;

    assert_eq!(v, 4);

    Ok(())
}
