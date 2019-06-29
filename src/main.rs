#![feature(async_await)]

use sqlx::{pg::Connection, ConnectOptions};
use std::io;

#[runtime::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let conn = Connection::establish(
        ConnectOptions::new()
            .host("127.0.0.1")
            .port(5433)
            .user("postgres")
            .password("password"),
    )
    .await?;

    //    conn.close().await?;

    Ok(())
}
