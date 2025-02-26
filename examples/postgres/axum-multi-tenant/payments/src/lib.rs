use accounts::AccountId;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::Type, Debug)]
#[sqlx(transparent)]
pub struct PaymentId(pub Uuid);

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "payments.payment_status")]
#[sqlx(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Successful,
}

#[derive(Debug)]
pub struct Payment {
    pub payment_id: PaymentId,
    pub account_id: AccountId,
    pub status: PaymentStatus,
    pub currency: String,
    // `rust_decimal::Decimal` has more than enough precision for any real-world amount of money.
    pub amount: rust_decimal::Decimal,
    pub external_payment_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
}

pub async fn migrate(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}
