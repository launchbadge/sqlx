use sqlx_core::{Connect, Connection, Tokio};
use sqlx_mysql::MySqlConnection;
use std::env;

#[tokio::test]
async fn it_connects() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let mut conn = MySqlConnection::<Tokio>::connect(&url).await?;

    conn.ping().await?;

    Ok(())
}
