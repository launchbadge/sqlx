use futures::TryStreamExt;
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
async fn it_can_select_expression() -> anyhow::Result<()> {
    let mut conn = new::<MsSql>().await?;

    let row: MsSqlRow = conn.fetch_one("SELECT 4").await?;
    let v: i32 = row.try_get(0)?;

    assert_eq!(v, 4);

    Ok(())
}

#[sqlx_macros::test]
async fn it_maths() -> anyhow::Result<()> {
    let mut conn = new::<MsSql>().await?;

    let value = sqlx::query("SELECT 1 + @p1")
        .bind(5_i32)
        .try_map(|row: MsSqlRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(6_i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<MsSql>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TABLE #users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO #users (id) VALUES (@p1)")
            .bind(index * 2)
            .execute(&mut conn)
            .await?;

        assert_eq!(cnt, 1);
    }

    let sum: i32 = sqlx::query("SELECT id FROM #users")
        .try_map(|row: MsSqlRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 110);

    Ok(())
}
