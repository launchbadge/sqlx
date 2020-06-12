use sqlx::MySql;
use sqlx_test::new;

#[sqlx_macros::test]
async fn macro_select_from_cte() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;
    let account =
        sqlx::query!("select * from (select (1) as id, 'Herp Derpinson' as name, cast(null as char) email) accounts")
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name, "Herp Derpinson");
    // MySQL can tell us the nullability of expressions, ain't that cool
    assert_eq!(account.email, None);

    Ok(())
}

#[sqlx_macros::test]
async fn macro_select_from_cte_bind() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;
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

#[derive(Debug)]
struct RawAccount {
    r#type: i32,
    name: Option<String>,
}

#[sqlx_macros::test]
async fn test_query_as_raw() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let account = sqlx::query_as!(
        RawAccount,
        "SELECT * from (select 1 as type, cast(null as char) as name) accounts"
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(account.name, None);
    assert_eq!(account.r#type, 1);

    println!("{:?}", account);

    Ok(())
}

#[sqlx_macros::test]
async fn test_query_as_bool() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    struct Article {
        id: i32,
        deleted: bool,
    }

    let article = sqlx::query_as_unchecked!(
        Article,
        "select * from (select 51 as id, true as deleted) articles"
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(51, article.id);
    assert_eq!(true, article.deleted);

    Ok(())
}

#[sqlx_macros::test]
async fn test_query_bytes() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let rec = sqlx::query!("SELECT X'01AF' as _1")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(rec._1, &[0x01_u8, 0xAF_u8]);

    Ok(())
}

#[sqlx_macros::test]
async fn test_column_override_not_null() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let record = sqlx::query!("select * from (select 1 as `id!`) records")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(record.id, 1);

    Ok(())
}

#[derive(PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(transparent)]
struct MyInt4(i32);

#[sqlx_macros::test]
async fn test_column_override_wildcard() -> anyhow::Result<()> {
    struct Record {
        id: MyInt4,
    }

    let mut conn = new::<MySql>().await?;

    let record = sqlx::query_as!(Record, "select * from (select 1 as `id: _`) records")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(record.id, MyInt4(1));

    Ok(())
}

#[sqlx_macros::test]
async fn test_column_override_exact() -> anyhow::Result<()> {
    let mut conn = new::<MySql>().await?;

    let record = sqlx::query!("select * from (select 1 as `id: MyInt4`) records")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(record.id, MyInt4(1));

    Ok(())
}

// we don't emit bind parameter typechecks for MySQL so testing the overrides is redundant
