// #[async_std::main]
// async fn main() -> anyhow::Result<()> {
//     let _stream = AsyncStd::connect_tcp("localhost", 5432).await?;
//
//     Ok(())
// }

use sqlx::mysql::MySqlConnectOptions;
use sqlx::prelude::*;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let mut conn = <MySqlConnection>::connect("mysql://").await?;
//
//     Ok(())
// }
//

// #[async_std::main]
// async fn main() -> anyhow::Result<()> {
//     let mut conn = <MySqlConnection>::builder()
//         .host("loca%x91lhost")
//         .port(20)
//         .connect()
//         .await?;
//
//     Ok(())
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut conn = <MySqlConnectOptions>::new().host("localhost").port(3306).connect().await?;

    Ok(())
}
