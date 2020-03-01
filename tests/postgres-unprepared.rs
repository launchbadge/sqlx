use sqlx::{Connect, Executor, Cursor, Row, PgConnection};

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_empty_query() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    let affected = conn.execute("").await?;

    assert_eq!(affected, 0);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_select_1() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    
    let mut cursor = conn.fetch("SELECT 5");
    let row = cursor.next().await?.unwrap();

    assert_eq!(5i32, row.get(0)?);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_multi_create_insert() -> anyhow::Result<()> {
    let mut conn = connect().await?;
    
    let mut cursor = conn.fetch("
CREATE TABLE IF NOT EXISTS _sqlx_test_postgres_5112 (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL
);

SELECT 'Hello World';

INSERT INTO _sqlx_test_postgres_5112 (text) VALUES ('this is a test');

SELECT id, text FROM _sqlx_test_postgres_5112;
    ");

    let row = cursor.next().await?.unwrap();

    assert!("Hello World" == row.get::<&str, _>(0)?);

    let row = cursor.next().await?.unwrap();

    assert_eq!(1_i64, row.get(0)?);
    assert!("this is a test" == row.get::<&str, _>(1)?);

    Ok(())
}

async fn connect() -> anyhow::Result<PgConnection> {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();

    Ok(PgConnection::connect(dotenv::var("DATABASE_URL")?).await?)
}
