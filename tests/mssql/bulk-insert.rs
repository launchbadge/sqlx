use sqlx::mssql::{IntoRow, Mssql};
use sqlx::Row;
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_bulk_inserts_rows() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    sqlx::query("CREATE TABLE #bulk_test (name NVARCHAR(50) NOT NULL, value INT NOT NULL)")
        .execute(&mut conn)
        .await?;

    let mut bulk = conn.bulk_insert("#bulk_test").await?;
    bulk.send(("hello", 1i32).into_row()).await?;
    bulk.send(("world", 2i32).into_row()).await?;
    bulk.send(("foo", 3i32).into_row()).await?;
    let total = bulk.finalize().await?;
    assert_eq!(total, 3);

    let rows = sqlx::query("SELECT name, value FROM #bulk_test ORDER BY value")
        .fetch_all(&mut conn)
        .await?;

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].get::<String, _>("name"), "hello");
    assert_eq!(rows[0].get::<i32, _>("value"), 1);
    assert_eq!(rows[1].get::<String, _>("name"), "world");
    assert_eq!(rows[1].get::<i32, _>("value"), 2);
    assert_eq!(rows[2].get::<String, _>("name"), "foo");
    assert_eq!(rows[2].get::<i32, _>("value"), 3);

    Ok(())
}

#[sqlx_macros::test]
async fn it_bulk_inserts_empty() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    sqlx::query("CREATE TABLE #bulk_empty (id INT NOT NULL)")
        .execute(&mut conn)
        .await?;

    let bulk = conn.bulk_insert("#bulk_empty").await?;
    let total = bulk.finalize().await?;
    assert_eq!(total, 0);

    Ok(())
}

#[sqlx_macros::test]
async fn it_bulk_inserts_various_types() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    sqlx::query(
        "CREATE TABLE #bulk_types (id INT NOT NULL, label NVARCHAR(100) NOT NULL, score FLOAT NOT NULL)"
    )
    .execute(&mut conn)
    .await?;

    let mut bulk = conn.bulk_insert("#bulk_types").await?;
    bulk.send((1i32, "alpha", 1.5f64).into_row()).await?;
    bulk.send((2i32, "beta", 2.7f64).into_row()).await?;
    let total = bulk.finalize().await?;
    assert_eq!(total, 2);

    let rows = sqlx::query("SELECT id, label, score FROM #bulk_types ORDER BY id")
        .fetch_all(&mut conn)
        .await?;

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get::<i32, _>("id"), 1);
    assert_eq!(rows[0].get::<String, _>("label"), "alpha");
    assert_eq!(rows[1].get::<i32, _>("id"), 2);
    assert_eq!(rows[1].get::<String, _>("label"), "beta");

    Ok(())
}
