use sqlx::types::Uuid;

#[async_std::test]
async fn postgres_query() -> sqlx::Result<()> {
    let mut conn =
        sqlx::Connection::<sqlx::Postgres>::open(&dotenv::var("DATABASE_URL").unwrap()).await?;

    let uuid: Uuid = "256ba9c8-0048-11ea-b0f0-8f04859d047e".parse().unwrap();
    let account_id = sqlx::query!("SELECT id from accounts where id != $1", uuid)
        .as_scalar::<Uuid>()
        .fetch_one(&mut conn)
        .await?;

    println!("account ID: {:?}", account_id);

    Ok(())
}
