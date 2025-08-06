use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

#[derive(sqlx::FromRow, Debug)]
pub struct Purchase {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

pub use rust_decimal::Decimal;
use uuid::Uuid;

pub async fn create_table(e: impl PgExecutor<'_>) -> sqlx::Result<()> {
    sqlx::raw_sql(
        // language=PostgreSQL
        "create table if not exists purchases( \
                id uuid primary key default gen_random_uuid(), \
                user_id uuid not null, \
                amount numeric not null check(amount > 0), \
                description text not null, \
                created_at timestamptz not null default now() \
             );
        ",
    )
    .execute(e)
    .await?;

    Ok(())
}

pub async fn create_purchase(
    e: impl PgExecutor<'_>,
    user_id: Uuid,
    amount: Decimal,
    description: &str,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar(
        "insert into purchases(user_id, amount, description) values ($1, $2, $3) returning id",
    )
    .bind(user_id)
    .bind(amount)
    .bind(description)
    .fetch_one(e)
    .await
}

pub async fn get_purchase(e: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<Option<Purchase>> {
    sqlx::query_as("select * from purchases where id = $1")
        .bind(id)
        .fetch_optional(e)
        .await
}
