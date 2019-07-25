#![feature(async_await)]

use futures::TryStreamExt;
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

    let row_id = conn
        .prepare("INSERT INTO users (name) VALUES ($1) RETURNING id")
        .bind(b"Joe")
        .get_result()
        .await?;

    println!("row_id: {:?}", row_id);

    let mut row_ids = conn.prepare("SELECT id FROM users").get_results();

    while let Some(row_id) = row_ids.try_next().await? {
        println!("row_ids: {:?}", row_id);
    }

    std::mem::drop(row_ids);

    let count = conn.prepare("SELECT name FROM users").execute().await?;
    println!("users: {}", count);

    conn.close().await?;

    Ok(())
}
