#![feature(async_await)]

use mason::{pg::Connection, ConnectOptions};

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::try_init()?;

    let mut conn =
        Connection::establish(ConnectOptions::new().user("postgres").password("password")).await?;

    conn.execute("INSERT INTO \"users\" (name) VALUES ($1)")
        .bind(b"Joe")
        .await?;

    conn.prepare("INSERT INTO \"users\" (name) VALUES ($1)")
        .bind(b"Joe")
        .execute()
        .await?;

    conn.close().await?;

    Ok(())
}
