#![feature(async_await)]

use futures::{future, TryStreamExt};
use sqlx::{postgres::Connection, ConnectOptions};
use std::io;

// TODO: ToSql and FromSql (to [de]serialize values from/to Rust and SQL)
// TODO: Connection strings ala postgres@localhost/sqlx_dev

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> io::Result<()> {
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

    println!(" :: drop database (if exists) sqlx__dev");

    conn.prepare("DROP DATABASE IF EXISTS sqlx__dev")
        .execute()
        .await?;

    println!(" :: create database sqlx__dev");

    conn.prepare("CREATE DATABASE sqlx__dev").execute().await?;

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

    let new_row = conn
        .prepare("INSERT INTO users (name) VALUES ($1) RETURNING id")
        .bind(b"Joe")
        .get()
        .await?;

    let new_id = new_row.as_ref().unwrap().get(0);

    println!("insert {:?}", new_id);

    println!(" :: select");

    conn.prepare("SELECT id FROM users")
        .select()
        .try_for_each(|row| {
            let id = row.get(0);

            println!("select {:?}", id);

            future::ok(())
        })
        .await?;

    conn.close().await?;

    Ok(())
}
