#![feature(async_await)]

use futures::{future, TryStreamExt};
use sqlx::{postgres::Connection, ConnectOptions};
use std::io;

// TODO: Connection strings ala postgres@localhost/sqlx_dev

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> io::Result<()> {
    env_logger::init();

    // Connect as postgres / postgres and DROP the sqlx__dev database
    // if exists and then re-create it
    let mut conn = Connection::establish(
        ConnectOptions::new()
            .host("127.0.0.1")
            .port(5432)
            .user("postgres")
            .database("postgres"),
    )
    .await?;

    println!(" :: create database sqlx__dev (if not exists)");

    conn.prepare("CREATE DATABASE IF NOT EXISTS sqlx__dev")
        .execute()
        .await?;

    conn.close().await?;

    let mut conn = Connection::establish(
        ConnectOptions::new()
            .host("127.0.0.1")
            .port(5432)
            .user("postgres")
            .database("sqlx__dev"),
    )
    .await?;

    println!(" :: create schema");

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

    println!(" :: insert");

    let user_id: i64 = conn
        .prepare("INSERT INTO users (name) VALUES ($1) RETURNING id")
        .bind("Joe")
        .get()
        .await?;

    println!("insert {:?}", user_id);

    println!(" :: select");

    conn.prepare("SELECT id, name FROM users")
        .select()
        .try_for_each(|(id, name): (i64, String)| {
            println!("select {} -> {}", id, name);

            future::ok(())
        })
        .await?;

    conn.close().await?;

    Ok(())
}
