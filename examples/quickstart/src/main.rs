// use sqlx::prelude::*;
use sqlx::blocking::{prelude::*, Blocking};
use sqlx::mysql::MySqlConnection;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     env_logger::try_init()?;
//
//     // connect to the database
//     let mut conn = <MySqlConnection>::connect("mysql://root:password@localhost").await?;
//
//     // ping, say HAI
//     conn.ping().await?;
//
//     // , and now close the connection explicitly
//     conn.close().await?;
//
//     Ok(())
// }

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    let mut conn = <MySqlConnection<Blocking>>::connect("mysql://root:password@localhost")?;

    conn.ping()?;
    conn.close()?;

    Ok(())
}
