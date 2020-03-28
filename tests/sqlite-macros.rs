use sqlx::Sqlite;
use sqlx_test::new;

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn macro_select() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let account = sqlx::query!("select id, name, is_active from accounts where id = 1")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(1, account.id);
    assert_eq!("Herp Derpinson", account.name);
    assert_eq!(account.is_active, None);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn macro_select_bind() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let account = sqlx::query!(
        "select id, name, is_active from accounts where id = ?",
        1i32
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(1, account.id);
    assert_eq!("Herp Derpinson", account.name);
    assert_eq!(account.is_active, None);

    Ok(())
}

#[derive(Debug)]
struct RawAccount {
    id: i32,
    name: String,
    is_active: Option<bool>,
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn test_query_as_raw() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let account = sqlx::query_as!(RawAccount, "SELECT id, name, is_active from accounts")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name, "Herp Derpinson");
    assert_eq!(account.is_active, None);

    Ok(())
}

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn macro_select_from_view() -> anyhow::Result<()> {
    let mut conn = new::<Sqlite>().await?;

    let account = sqlx::query!("SELECT id, name, is_active from accounts_view")
        .fetch_one(&mut conn)
        .await?;

    // SQLite tells us the true origin of these columns even through the view
    assert_eq!(account.id, 1);
    assert_eq!(account.name, "Herp Derpinson");
    assert_eq!(account.is_active, None);

    Ok(())
}
