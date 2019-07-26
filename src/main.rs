#![feature(async_await)]

use futures::{future, TryStreamExt};
use sqlx::{postgres::Connection, ConnectOptions};
use std::io;

// TODO: ToSql and FromSql (to [de]serialize values from/to Rust and SQL)
// TODO: Connection strings ala postgres@localhost/sqlx_dev

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

    let new_id = conn
        .prepare("INSERT INTO users (name) VALUES ($1) RETURNING id")
        .bind(b"Joe")
        .get()
        .await?;

    println!("insert {:?}", new_id);

    conn.prepare("SELECT id FROM users")
        .select()
        .try_for_each(|row| {
            println!("select {:?}", row.get(0));

            future::ok(())
        })
        .await?;

    conn.close().await?;

    Ok(())
}
