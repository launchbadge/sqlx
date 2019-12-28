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
async fn it_connects_to_database_user() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let row = sqlx::query("select current_database()")
        .fetch_one(&mut conn)
        .await?;

    let current_db: String = row.get(0);

    let row = sqlx::query("select current_user")
        .fetch_one(&mut conn)
        .await?;

    let current_user: String = row.get(0);

    assert_eq!(current_db, "postgres");
    assert_eq!(current_user, "postgres");

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

async fn connect() -> anyhow::Result<PgConnection> {
    Ok(PgConnection::open(dotenv::var("DATABASE_URL")?).await?)
}
