use sqlx::mysql::MySqlConnection;
use sqlx::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _conn = <MySqlConnection>::connect("mysql://root:password@localhost:3307").await?;

    Ok(())
}
