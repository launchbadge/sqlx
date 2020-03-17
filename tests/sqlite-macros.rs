use sqlx::Sqlite;
use sqlx_test::new;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn macro_select() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;
    let account = sqlx::query!("select * from accounts")
        .fetch_one(&mut conn)
        .await?;

    println!("{:?}", account);
    println!("{}: {}", account.id, account.name);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn macro_select_bind() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;
    let account = sqlx::query!("select * from accounts where id = ?", 1i32)
        .fetch_one(&mut conn)
        .await?;

    println!("{:?}", account);
    println!("{}: {}", account.id, account.name);

    Ok(())
}

#[derive(Debug)]
struct RawAccount {
    id: i32,
    name: String,
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_query_as_raw() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let account = sqlx::query_as!(RawAccount, "SELECT * from accounts")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(1, account.id);
    assert_eq!("Herp Derpinson", account.name);

    println!("{:?}", account);

    Ok(())
}
