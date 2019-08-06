#![feature(async_await)]

use futures::{future, TryStreamExt};
use sqlx::{postgres::Connection, ConnectOptions};
use std::{collections::HashMap, io};

// TODO: Connection strings ala postgres@localhost/sqlx_dev

#[runtime::main(runtime_tokio::Tokio)]
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

    println!(" :: create schema");

    conn.prepare(
        r#"
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    password TEXT
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

    // Iterate through each row, one-by-one
    conn.prepare("SELECT id, name, password FROM users")
        .select()
        .try_for_each(|(id, name, password): (i64, String, Option<String>)| {
            println!("select {} -> {} (password = {:?})", id, name, password);

            future::ok(())
        })
        .await?;

    // Get a map of id -> name of users with a name of JOE
    let map: HashMap<i64, String> = conn
        .prepare("SELECT id, name FROM users WHERE name = $1")
        .bind("Joe")
        .select()
        .try_collect()
        .await?;

    println!(" :: users\n{:#?}\n", map);

    conn.close().await?;

    Ok(())
}
