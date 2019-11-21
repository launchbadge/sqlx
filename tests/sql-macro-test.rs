#![feature(proc_macro_hygiene)]

#[async_std::test]
async fn test_sqlx_macro() -> sqlx::Result<()> {
    let conn =
        sqlx::Connection::<sqlx::Postgres>::establish("postgres://postgres@127.0.0.1/sqlx_test")
            .await?;
    let uuid: sqlx::types::Uuid = "256ba9c8-0048-11ea-b0f0-8f04859d047e".parse().unwrap();
    let accounts = sqlx_macros::sql!("SELECT * from accounts where id = $1", None)
        .fetch_one(&conn)
        .await?;

    println!("accounts: {:?}", accounts);

    Ok(())
}
