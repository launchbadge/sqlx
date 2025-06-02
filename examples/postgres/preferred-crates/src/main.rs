use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Connection, PgConnection};
use std::time::Duration;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
struct SessionData {
    user_id: Uuid,
}

#[derive(sqlx::FromRow, Debug)]
struct User {
    id: Uuid,
    username: String,
    password_hash: String,
    // Because `time` is enabled by a transitive dependency, we previously would have needed
    // a type override in the query to get types from `chrono`.
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

const SESSION_DURATION: Duration = Duration::from_secs(60 * 60); // 1 hour

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut conn =
        PgConnection::connect(&dotenvy::var("DATABASE_URL").context("DATABASE_URL must be set")?)
            .await
            .context("failed to connect to DATABASE_URL")?;

    sqlx::migrate!("./src/migrations").run(&mut conn).await?;

    uses_rust_decimal::create_table(&mut conn).await?;
    uses_time::create_table(&mut conn).await?;

    let user_id = sqlx::query_scalar!(
        "insert into users(username, password_hash) values($1, $2) returning id",
        "user_foo",
        "<pretend this is a password hash>",
    )
    .fetch_one(&mut conn)
    .await?;

    let user = sqlx::query_as!(User, "select * from users where id = $1", user_id)
        .fetch_one(&mut conn)
        .await?;

    println!("Created user: {user:?}");

    let session =
        uses_time::create_session(&mut conn, SessionData { user_id }, SESSION_DURATION).await?;

    let session_from_id = uses_time::get_session::<SessionData>(&mut conn, session.id)
        .await?
        .expect("expected session");

    assert_eq!(session, session_from_id);

    let purchase_id =
        uses_rust_decimal::create_purchase(&mut conn, user_id, 1234u32.into(), "Rent").await?;

    let purchase = uses_rust_decimal::get_purchase(&mut conn, purchase_id)
        .await?
        .expect("expected purchase");

    println!("Created purchase: {purchase:?}");

    Ok(())
}
