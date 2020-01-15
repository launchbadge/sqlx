use sqlx::{PgConnection, Connect};

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_query() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let account = sqlx::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
        1i32
    )
    .fetch_one(&mut conn)
    .await?;

    println!("account ID: {:?}", account.id);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_query_file() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let account = sqlx::query_file!("tests/test-query.sql")
        .fetch_one(&mut conn)
        .await?;

    println!("{:?}", account);

    Ok(())
}

#[derive(Debug)]
struct Account {
    id: i32,
    name: Option<String>,
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_query_as() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let account = sqlx::query_as!(
        Account,
        "SELECT * from (VALUES (1, null)) accounts(id, name)"
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(None, account.name);

    println!("{:?}", account);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_query_file_as() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let account = sqlx::query_file_as!(Account, "tests/test-query.sql")
        .fetch_one(&mut conn)
        .await?;

    println!("{:?}", account);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn query_by_string() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let string = "Hello, world!".to_string();

    let result = sqlx::query!(
        "SELECT * from (VALUES('Hello, world!')) strings(string)\
         where string = $1 or string = $2",
        string,
        string[..]
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(result.string, string);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_nullable_err() -> anyhow::Result<()> {
    #[derive(Debug)]
    struct Account {
        id: i32,
        name: String,
    }

    let mut conn = connect().await?;

    let err = sqlx::query_as!(
        Account,
        "SELECT * from (VALUES (1, null::text)) accounts(id, name)"
    )
    .fetch_one(&mut conn)
    .await
    .unwrap_err();

    if let sqlx::Error::Decode(sqlx::decode::DecodeError::UnexpectedNull) = err {
        Ok(())
    } else {
        panic!("expected `UnexpectedNull`, got {}", err)
    }
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_many_args() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    // previous implementation would only have supported 10 bind parameters
    // (this is really gross to test in MySQL)
    let rows = sqlx::query!(
        "SELECT * from unnest(array[$1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12]::int[]) ids(id);",
        0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32, 10i32, 11i32
    )
        .fetch_all(&mut conn)
        .await?;

    for (i, row) in rows.iter().enumerate() {
        assert_eq!(i as i32, row.id);
    }

    Ok(())
}

async fn connect() -> anyhow::Result<PgConnection> {
    let _ = dotenv::dotenv();
    let _ = env_logger::try_init();

    Ok(PgConnection::connect(dotenv::var("DATABASE_URL")?).await?)
}
