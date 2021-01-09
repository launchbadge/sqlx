use sqlx::mysql::MySqlConnection;
use sqlx::prelude::*;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     env_logger::try_init()?;
//
//     let _conn = <MySqlConnection>::connect("mysql://root:password@localhost").await?;
//
//     Ok(())
// }

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    let _conn = <MySqlConnection>::connect("mysql://root:password@localhost")?;

    Ok(())
}
