#![feature(async_await)]

use sqlx::{pg::Connection, ConnectOptions};
use std::io;

// TODO: ToSql and FromSql (to [de]serialize values from/to Rust and SQL)
// TODO: Connection strings ala postgres@localhost/sqlx_dev
// TODO: Queries (currently we only support EXECUTE [drop results])

#[runtime::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let mut conn = Connection::establish(
        ConnectOptions::new()
            .host("127.0.0.1")
            .port(5432)
            .user("postgres")
            .database("sqlx__dev"),
    )
    .await?;

    conn.prepare(
        r#"
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
        "#,
    )
    .execute()
    .await?;

    conn.prepare("INSERT INTO users (name) VALUES ('George')")
        // .bind(b"Joe")
        .execute()
        .await?;

    let count = conn.prepare("SELECT name FROM users").execute().await?;
    println!("users: {}", count);

    conn.close().await?;

    Ok(())
}
