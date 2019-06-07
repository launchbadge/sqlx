#![feature(async_await)]

use mason::pg::Connection;

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::try_init()?;

    let mut conn = Connection::open("127.0.0.1:5432").await?;

    conn.startup("postgres", "", "postgres").await?;
    conn.terminate().await?;

    Ok(())
}
