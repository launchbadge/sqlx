use futures::TryStreamExt;
use sqlx::{postgres::PgConnection, Connection as _, Executor as _, Row as _};

#[async_std::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let row = sqlx::query("select 1 + 1").fetch_one(&mut conn).await?;

    assert_eq!(2, row.get(0));

    conn.close().await?;

    Ok(())
}

#[async_std::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let _ = conn
        .send(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO users (id) VALUES ($1)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(cnt, 1);
    }

    let sum: i32 = sqlx::query("SELECT id FROM users")
        .fetch(&mut conn)
        .try_fold(
            0_i32,
            |acc, x| async move { Ok(acc + x.get::<i32, _>("id")) },
        )
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

#[async_std::test]
async fn it_remains_stable_issue_30() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    // This tests the internal buffer wrapping around at the end
    // Specifically: https://github.com/launchbadge/sqlx/issues/30

    let rows = sqlx::query("SELECT i, random()::text FROM generate_series(1, 1000) as i")
        .fetch_all(&mut conn)
        .await?;

    assert_eq!(rows.len(), 1000);
    assert_eq!(rows[rows.len() - 1].get::<i32, _>(0), 1000);

    Ok(())
}

async fn connect() -> anyhow::Result<PgConnection> {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();
    Ok(PgConnection::open(dotenv::var("DATABASE_URL")?).await?)
}
