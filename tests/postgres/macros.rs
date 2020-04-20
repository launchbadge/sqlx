use sqlx::Postgres;
use sqlx_test::new;

use futures::TryStreamExt;

#[sqlx_rt::test]
async fn test_query() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let account = sqlx::query!(
        "SELECT * from (VALUES (1, 'Herp Derpinson')) accounts(id, name) where id = $1",
        1i32
    )
    .fetch_one(&mut conn)
    .await?;

    println!("account ID: {:?}", account.id);

    Ok(())
}

#[sqlx_rt::test]
async fn test_no_result() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let _ = sqlx::query!("DELETE FROM pg_enum")
        .execute(&mut conn)
        .await?;

    Ok(())
}

#[sqlx_rt::test]
async fn test_text_var_char_char_n() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // TEXT
    // we cannot infer nullability from an expression
    let rec = sqlx::query!("SELECT 'Hello'::text as greeting")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(rec.greeting.as_deref(), Some("Hello"));

    // VARCHAR(N)

    let rec = sqlx::query!("SELECT 'Hello'::varchar(5) as greeting")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(rec.greeting.as_deref(), Some("Hello"));

    // CHAR(N)

    let rec = sqlx::query!("SELECT 'Hello'::char(5) as greeting")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(rec.greeting.as_deref(), Some("Hello"));

    Ok(())
}

#[sqlx_rt::test]
async fn _file() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let account = sqlx::query_file!("tests/test-query.sql",)
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

#[sqlx_rt::test]
async fn test_query_as() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let name: Option<&str> = None;
    let account = sqlx::query_as!(
        Account,
        "SELECT * from (VALUES (1, $1)) accounts(id, name)",
        name
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(None, account.name);

    println!("{:?}", account);

    Ok(())
}

#[derive(Debug)]
struct RawAccount {
    r#type: i32,
    name: Option<String>,
}

#[sqlx_rt::test]
async fn test_query_as_raw() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let account = sqlx::query_as!(
        RawAccount,
        "SELECT * from (VALUES (1, null)) accounts(type, name)"
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(None, account.name);
    assert_eq!(1, account.r#type);

    println!("{:?}", account);

    Ok(())
}

#[sqlx_rt::test]
async fn test_query_file_as() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let account = sqlx::query_file_as!(Account, "tests/test-query.sql",)
        .fetch_one(&mut conn)
        .await?;

    println!("{:?}", account);

    Ok(())
}

#[sqlx_rt::test]
async fn query_by_string() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let string = "Hello, world!".to_string();
    let ref tuple = ("Hello, world!".to_string(),);

    let result = sqlx::query!(
        "SELECT * from (VALUES('Hello, world!')) strings(string)\
         where string in ($1, $2, $3, $4, $5, $6, $7)",
        string, // make sure we don't actually take ownership here
        &string[..],
        Some(&string),
        Some(&string[..]),
        Option::<String>::None,
        string.clone(),
        tuple.0 // make sure we're not trying to move out of a field expression
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(result.string, Some(string));

    Ok(())
}

#[sqlx_rt::test]
async fn test_nullable_err() -> anyhow::Result<()> {
    #[derive(Debug)]
    struct Account {
        id: i32,
        name: String,
    }

    let mut conn = new::<Postgres>().await?;

    let err = sqlx::query_as!(
        Account,
        "SELECT * from (VALUES (1, null::text)) accounts(id, name)"
    )
    .fetch_one(&mut conn)
    .await
    .unwrap_err();

    if let sqlx::Error::Decode(err) = &err {
        if let Some(sqlx::error::UnexpectedNullError) = err.downcast_ref() {
            return Ok(());
        }
    }

    panic!("expected `UnexpectedNullError`, got {}", err)
}

#[sqlx_rt::test]
async fn test_many_args() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // previous implementation would only have supported 10 bind parameters
    // (this is really gross to test in MySQL)
    let rows = sqlx::query!(
        "SELECT * from unnest(array[$1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12]::int[]) ids(id);",
        0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 6i32, 7i32, 8i32, 9i32, 10i32, 11i32
    )
        .fetch_all(&mut conn)
        .await?;

    for (i, row) in rows.iter().enumerate() {
        assert_eq!(Some(i as i32), row.id);
    }

    Ok(())
}

#[sqlx_rt::test]
async fn test_array_from_slice() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let list: &[i32] = &[1, 2, 3, 4i32];

    let result = sqlx::query!("SELECT $1::int[] as my_array", list)
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(result.my_array, Some(vec![1, 2, 3, 4]));

    println!("result ID: {:?}", result.my_array);

    let account = sqlx::query!("SELECT ARRAY[4,3,2,1] as my_array")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(account.my_array, Some(vec![4, 3, 2, 1]));

    println!("account ID: {:?}", account.my_array);

    Ok(())
}

#[sqlx_rt::test]
async fn fetch_is_usable_issue_224() -> anyhow::Result<()> {
    // ensures that the stream returned by `query::Map::fetch()` is usable with `TryStreamExt`
    let mut conn = new::<Postgres>().await?;

    let mut stream =
        sqlx::query!("select * from generate_series(1, 3) as series(num);").fetch(&mut conn);

    // `num` is generated by a function so we can't assume it's non-null
    assert_eq!(stream.try_next().await?.unwrap().num, Some(1));
    assert_eq!(stream.try_next().await?.unwrap().num, Some(2));
    assert_eq!(stream.try_next().await?.unwrap().num, Some(3));
    assert!(stream.try_next().await?.is_none());

    Ok(())
}
