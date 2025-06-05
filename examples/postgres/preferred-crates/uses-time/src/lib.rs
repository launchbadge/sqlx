use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::PgExecutor;
use std::time::Duration;
use time::OffsetDateTime;

use sqlx::types::Json;
use uuid::Uuid;

#[derive(sqlx::FromRow, PartialEq, Eq, Debug)]
pub struct Session<D> {
    pub id: Uuid,
    #[sqlx(json)]
    pub data: D,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

pub async fn create_table(e: impl PgExecutor<'_>) -> sqlx::Result<()> {
    sqlx::raw_sql(
        // language=PostgreSQL
        "create table if not exists sessions( \
                id uuid primary key default gen_random_uuid(), \
                data jsonb not null,
                created_at timestamptz not null default now(),
                expires_at timestamptz not null
             )",
    )
    .execute(e)
    .await?;

    Ok(())
}

pub async fn create_session<D: Serialize>(
    e: impl PgExecutor<'_>,
    data: D,
    valid_duration: Duration,
) -> sqlx::Result<Session<D>> {
    // Round down to the nearest second because
    // Postgres doesn't support precision higher than 1 microsecond anyway.
    let created_at = OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("0 nanoseconds should be in range");

    let expires_at = created_at + valid_duration;

    let id: Uuid = sqlx::query_scalar(
        "insert into sessions(data, created_at, expires_at) \
             values ($1, $2, $3) \
             returning id",
    )
    .bind(Json(&data))
    .bind(created_at)
    .bind(expires_at)
    .fetch_one(e)
    .await?;

    Ok(Session {
        id,
        data,
        created_at,
        expires_at,
    })
}

pub async fn get_session<D: DeserializeOwned + Send + Unpin + 'static>(
    e: impl PgExecutor<'_>,
    id: Uuid,
) -> sqlx::Result<Option<Session<D>>> {
    sqlx::query_as("select id, data, created_at, expires_at from sessions where id = $1")
        .bind(id)
        .fetch_optional(e)
        .await
}
