use futures::TryStreamExt;
use sqlx::{Connection as _, Executor as _, MySqlConnection, MySqlPool, Row as _};

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
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY)
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO users (id) VALUES (?)")
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
async fn pool_immediately_fails_with_db_error() -> anyhow::Result<()> {
    // Malform the database url by changing the password
    let url = url()?.replace("password", "not-the-password");

    let pool = MySqlPool::new(&url).await?;

    let res = pool.acquire().await;

    match res {
        Err(sqlx::Error::Database(err)) if err.message().contains("Access denied") => {
            // Access was properly denied
        }

        Err(e) => panic!("unexpected error: {:?}", e),

        Ok(_) => panic!("unexpected ok"),
    }

    Ok(())
}

#[cfg(feature = "macros")]
#[async_std::test]
async fn macro_select_from_cte() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    let account =
        sqlx::query!("select * from (select (1) as id, 'Herp Derpinson' as name) accounts")
            .fetch_one(&mut conn)
            .await?;

    println!("{:?}", account);
    println!("{}: {}", account.id, account.name);

    Ok(())
}

#[cfg(feature = "macros")]
#[async_std::test]
async fn macro_select_from_cte_bind() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    let account = sqlx::query!(
        "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
        1i32
    )
    .fetch_one(&mut conn)
    .await?;

    println!("{:?}", account);
    println!("{}: {}", account.id, account.name);

    Ok(())
}

fn url() -> anyhow::Result<String> {
    Ok(dotenv::var("DATABASE_URL")?)
}

async fn connect() -> anyhow::Result<MySqlConnection> {
    Ok(MySqlConnection::open(url()?).await?)
}
