#[async_std::test]
async fn test_query() -> sqlx::Result<()> {
    let mut conn = sqlx::postgres::connect(&dotenv::var("DATABASE_URL").unwrap()).await?;

    let account = sqlx::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
        1i32
    )
    .fetch_one(&mut conn)
    .await?;

    println!("account ID: {:?}", account.id);

    Ok(())
}

#[async_std::test]
async fn test_query_file() -> sqlx::Result<()> {
    let mut conn = sqlx::postgres::connect(&dotenv::var("DATABASE_URL").unwrap()).await?;

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

#[async_std::test]
async fn test_query_as() -> sqlx::Result<()> {
    let mut conn = sqlx::postgres::connect(&dotenv::var("DATABASE_URL").unwrap()).await?;

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

#[async_std::test]
async fn test_query_file_as() -> sqlx::Result<()> {
    let mut conn = sqlx::postgres::connect(&dotenv::var("DATABASE_URL").unwrap()).await?;

    let account = sqlx::query_file_as!(Account, "tests/test-query.sql")
        .fetch_one(&mut conn)
        .await?;

    println!("{:?}", account);

    Ok(())
}

#[async_std::test]
async fn test_nullable_err() -> sqlx::Result<()> {
    #[derive(Debug)]
    struct Account {
        id: i32,
        name: String,
    }

    let mut conn = sqlx::postgres::connect(&dotenv::var("DATABASE_URL").unwrap()).await?;

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
