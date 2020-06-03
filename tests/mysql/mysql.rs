use futures::TryStreamExt;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::{Connection, Executor, Row};
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    conn.ping().await?;
    conn.close().await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_maths() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let value = sqlx::query("select 1 + CAST(? AS SIGNED)")
        .bind(5_i32)
        .try_map(|row: MySqlRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(6i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_at_querying() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let _ = conn.execute(sqlx::query("SELECT 1")).await?;

    // we are testing that this does not cause a panic!
    let _ = conn
        .execute(sqlx::query("SELECT non_existence_table"))
        .await;

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY);
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
        .try_map(|row: MySqlRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

#[sqlx_macros::test]
async fn it_drops_results_in_affected_rows() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    // ~1800 rows should be iterated and dropped
    let affected = conn
        .execute("select * from mysql.time_zone limit 1575")
        .await?;

    // In MySQL, rows being returned isn't enough to flag it as an _affected_ row
    assert_eq!(0, affected);

    Ok(())
}

#[sqlx_macros::test]
async fn it_selects_null() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let (val,): (Option<i32>,) = sqlx::query_as("SELECT NULL").fetch_one(&mut conn).await?;

    assert!(val.is_none());

    let val: Option<i32> = conn.fetch_one("SELECT NULL").await?.try_get(0)?;

    assert!(val.is_none());

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fetch_one_and_ping() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let (_id,): (i32,) = sqlx::query_as("SELECT 1 as id")
        .fetch_one(&mut conn)
        .await?;

    conn.ping().await?;

    let (_id,): (i32,) = sqlx::query_as("SELECT 1 as id")
        .fetch_one(&mut conn)
        .await?;

    Ok(())
}

/// Test that we can interleave reads and writes to the database in one simple query.
#[sqlx_macros::test]
async fn it_interleaves_reads_and_writes() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let mut s = conn.fetch(
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

    let row = s.try_next().await?.unwrap();

    assert!("Hello World" == row.try_get::<&str, _>("_1")?);

    let row = s.try_next().await?.unwrap();

    let id: i64 = row.try_get("id")?;
    let text: &str = row.try_get("text")?;

    assert_eq!(1_i64, id);
    assert_eq!("this is a test", text);

    Ok(())
}
