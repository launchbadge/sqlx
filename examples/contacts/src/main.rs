#![feature(async_await)]

use failure::Fallible;
use fake::{
    faker::{
        internet::en::{Password, SafeEmail, Username},
        name::en::Name,
        phone_number::en::PhoneNumber,
    },
    Dummy, Fake, Faker,
};
use futures::future;
use sqlx::{pool::Pool, postgres::Postgres};

#[derive(Debug, Dummy)]
struct Contact {
    #[dummy(faker = "Name()")]
    name: String,

    #[dummy(faker = "Username()")]
    username: String,

    #[dummy(faker = "Password(5..25)")]
    password: String,

    #[dummy(faker = "SafeEmail()")]
    email: String,

    #[dummy(faker = "PhoneNumber()")]
    phone: String,
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Fallible<()> {
    env_logger::try_init()?;

    let options = sqlx::ConnectOptions::new()
        .host("127.0.0.1")
        .port(5432)
        .user("postgres")
        .database("sqlx__dev__contacts");

    let pool = Pool::<Postgres>::new(options);

    {
        let mut conn = pool.acquire().await?;
        conn.prepare(
            r#"
CREATE TABLE IF NOT EXISTS contacts (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    name TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL
)
            "#,
        )
        .execute()
        .await?;
    }

    let mut handles = vec![];

    for _ in 0..50_000 {
        let pool = pool.clone();
        let contact: Contact = Faker.fake();
        let handle: runtime::task::JoinHandle<Fallible<()>> = runtime::task::spawn(async move {
            let mut conn = pool.acquire().await?;
            conn.prepare(
                r#"
    INSERT INTO contacts (name, username, password, email, phone)
    VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(contact.name)
            .bind(contact.username)
            .bind(contact.password)
            .bind(contact.email)
            .bind(contact.phone)
            .execute()
            .await?;

            Ok(())
        });

        handles.push(handle);
    }

    future::join_all(handles).await;

    Ok(())
}
