use accounts::{AccountId, AccountsManager};
use sqlx::{Acquire, PgConnection, PgPool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

#[derive(sqlx::Type, Copy, Clone, Debug)]
#[sqlx(transparent)]
pub struct PaymentId(pub Uuid);

#[derive(sqlx::Type, Copy, Clone, Debug)]
#[sqlx(type_name = "payment_status")]
#[sqlx(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Created,
    Success,
    Failed,
}

// Users often assume that they need `#[derive(FromRow)]` to use `query_as!()`,
// then are surprised when the derive's control attributes have no effect.
// The macros currently do *not* use the `FromRow` trait at all.
// Support for `FromRow` is planned, but would require significant changes to the macros.
// See https://github.com/launchbadge/sqlx/issues/514 for details.
#[derive(Clone, Debug)]
pub struct Payment {
    pub payment_id: PaymentId,
    pub account_id: AccountId,
    pub status: PaymentStatus,
    pub currency: String,
    // `rust_decimal::Decimal` has more than enough precision for any real-world amount of money.
    pub amount: rust_decimal::Decimal,
    pub external_payment_id: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
}

pub struct PaymentsManager {
    pool: PgPool,
}

impl PaymentsManager {
    pub async fn setup(
        opts: PgConnectOptions,
    ) -> sqlx::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        sqlx::migrate!()
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }

    /// # Note
    /// For simplicity, this does not ensure that `account_id` actually exists.
    pub async fn create(
        &self,
        account_id: AccountId,
                            currency: &str,
                            amount: rust_decimal::Decimal,
    ) -> sqlx::Result<Payment> {
        // Check-out a connection to avoid paying the overhead of acquiring one for each call.
        let mut conn = self.pool.acquire().await?;

        // Imagine this method does more than just create a record in the database;
        // maybe it actually initiates the payment with a third-party vendor, like Stripe.
        //
        // We need to ensure that we can link the payment in the vendor's systems back to a record
        // in ours, even if any of the following happens:
        // * The application dies before storing the external payment ID in the database
        // * We lose the connection to the database while trying to commit a transaction
        // * The database server dies while committing the transaction
        //
        // Thus, we create the payment in three atomic phases:
        // * We create the payment record in our system and commit it.
        // * We create the payment in the vendor's system with our payment ID attached.
        // * We update our payment record with the vendor's payment ID.
        let payment_id = sqlx::query_scalar!(
            "insert into payment(account_id, status, currency, amount) \
             values ($1, $2, $3, $4) \
             returning payment_id",
            // The database doesn't give us enough information to correctly typecheck `AccountId` here.
            // We have to insert the UUID directly.
            account_id.0,
            PaymentStatus::Pending,
            currency,
            amount,
        )
            .fetch_one(&mut *conn)
            .await?;

        // We then create the record with the payment vendor...
        let external_payment_id = "foobar1234";

        // Then we store the external payment ID and update the payment status.
        //
        // NOTE: use caution with `select *` or `returning *`;
        // the order of columns gets baked into the binary, so if it changes between compile time and
        // run-time, you may run into errors.
        let payment = sqlx::query_as!(
        Payment,
        "update payment \
         set status = $1, external_payment_id = $2 \
         where payment_id = $3 \
         returning *",
        PaymentStatus::Created,
        external_payment_id,
        payment_id.0,
    )
            .fetch_one(&mut *conn)
            .await?;

        Ok(payment)
    }

    pub async fn get(&self, payment_id: PaymentId) -> sqlx::Result<Option<Payment>> {
        sqlx::query_as!(
            Payment,
            // see note above about `select *`
            "select * from payment where payment_id = $1",
            payment_id.0
        )
            .fetch_optional(&self.pool)
            .await
    }
}


