use argon2::{password_hash, Argon2, PasswordHasher, PasswordVerifier};
use std::sync::Arc;

use password_hash::PasswordHashString;

use sqlx::{PgConnection, PgPool, PgTransaction};

use uuid::Uuid;

use tokio::sync::Semaphore;

#[derive(sqlx::Type, Debug)]
#[sqlx(transparent)]
pub struct AccountId(pub Uuid);

pub struct AccountsManager {
    /// Controls how many blocking tasks are allowed to run concurrently for Argon2 hashing.
    ///
    /// ### Motivation
    /// Tokio blocking tasks are generally not designed for CPU-bound work.
    ///
    /// If no threads are idle, Tokio will automatically spawn new ones to handle
    /// new blocking tasks up to a very high limit--512 by default.
    ///
    /// This is because blocking tasks are expected to spend their time *blocked*, e.g. on
    /// blocking I/O, and thus not consume CPU resources or require a lot of context switching.
    ///
    /// This strategy is not the most efficient way to use threads for CPU-bound work, which
    /// should schedule work to a fixed number of threads to minimize context switching
    /// and memory usage (each new thread needs significant space allocated for its stack).
    ///
    /// We can work around this by using a purpose-designed thread-pool, like Rayon,
    /// but we still have the problem that those APIs usually are not designed to support `async`,
    /// so we end up needing blocking tasks anyway, or implementing our own work queue using
    /// channels. Rayon also does not shut down idle worker threads.
    ///
    /// `block_in_place` is not a silver bullet, either, as it simply uses `spawn_blocking`
    /// internally to take over from the current thread while it is executing blocking work.
    /// This also prevents futures from being polled concurrently in the current task.
    ///
    /// We can lower the limit for blocking threads when creating the runtime, but this risks
    /// starving other blocking tasks that are being created by the application or the Tokio
    /// runtime itself
    /// (which are used for `tokio::fs`, stdio, resolving of hostnames by `ToSocketAddrs`, etc.).
    ///
    /// Instead, we can just use a Semaphore to limit how many blocking tasks are spawned at once,
    /// emulating the behavior of a thread pool like Rayon without needing any additional crates.
    hashing_semaphore: Arc<Semaphore>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("error creating account: email in-use")]
    EmailInUse,
    #[error("error creating account")]
    General(
        #[source]
        #[from]
        GeneralError,
    ),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticateError {
    #[error("unknown email")]
    UnknownEmail,
    #[error("invalid password")]
    InvalidPassword,
    #[error("authentication error")]
    General(
        #[source]
        #[from]
        GeneralError,
    ),
}

#[derive(Debug, thiserror::Error)]
pub enum GeneralError {
    #[error("database error")]
    Sqlx(
        #[source]
        #[from]
        sqlx::Error,
    ),
    #[error("error hashing password")]
    PasswordHash(
        #[source]
        #[from]
        password_hash::Error,
    ),
    #[error("task panicked")]
    Task(
        #[source]
        #[from]
        tokio::task::JoinError,
    ),
}

impl AccountsManager {
    pub async fn setup(pool: &PgPool, max_hashing_threads: usize) -> Result<Self, GeneralError> {
        sqlx::migrate!()
            .run(pool)
            .await
            .map_err(sqlx::Error::from)?;

        Ok(AccountsManager {
            hashing_semaphore: Semaphore::new(max_hashing_threads).into(),
        })
    }

    async fn hash_password(&self, password: String) -> Result<PasswordHashString, GeneralError> {
        let guard = self
            .hashing_semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("BUG: this semaphore should not be closed");

        // We transfer ownership to the blocking task and back to ensure Tokio doesn't spawn
        // excess threads.
        let (_guard, res) = tokio::task::spawn_blocking(move || {
            let salt = argon2::password_hash::SaltString::generate(rand::thread_rng());
            (
                guard,
                Argon2::default()
                    .hash_password(password.as_bytes(), &salt)
                    .map(|hash| hash.serialize()),
            )
        })
        .await?;

        Ok(res?)
    }

    async fn verify_password(
        &self,
        password: String,
        hash: PasswordHashString,
    ) -> Result<(), AuthenticateError> {
        let guard = self
            .hashing_semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("BUG: this semaphore should not be closed");

        let (_guard, res) = tokio::task::spawn_blocking(move || {
            (
                guard,
                Argon2::default().verify_password(password.as_bytes(), &hash.password_hash()),
            )
        })
        .await
        .map_err(GeneralError::from)?;

        if let Err(password_hash::Error::Password) = res {
            return Err(AuthenticateError::InvalidPassword);
        }

        res.map_err(GeneralError::from)?;

        Ok(())
    }

    pub async fn create(
        &self,
        txn: &mut PgTransaction<'_>,
        email: &str,
        password: String,
    ) -> Result<AccountId, CreateError> {
        // Hash password whether the account exists or not to make it harder
        // to tell the difference in the timing.
        let hash = self.hash_password(password).await?;

        // Thanks to `sqlx.toml`, `account_id` maps to `AccountId`
        sqlx::query_scalar!(
            // language=PostgreSQL
            "insert into accounts.account(email, password_hash) \
             values ($1, $2) \
             returning account_id",
            email,
            hash.as_str(),
        )
        .fetch_one(&mut **txn)
        .await
        .map_err(|e| {
            if e.as_database_error().and_then(|dbe| dbe.constraint())
                == Some("account_account_id_key")
            {
                CreateError::EmailInUse
            } else {
                GeneralError::from(e).into()
            }
        })
    }

    pub async fn authenticate(
        &self,
        conn: &mut PgConnection,
        email: &str,
        password: String,
    ) -> Result<AccountId, AuthenticateError> {
        // Thanks to `sqlx.toml`:
        // * `account_id` maps to `AccountId`
        // * `password_hash` maps to `Text<PasswordHashString>`
        let maybe_account = sqlx::query!(
            "select account_id, password_hash \
             from accounts.account \
             where email = $1",
            email
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(GeneralError::from)?;

        let Some(account) = maybe_account else {
            // Hash the password whether the account exists or not to hide the difference in timing.
            self.hash_password(password)
                .await
                .map_err(GeneralError::from)?;
            return Err(AuthenticateError::UnknownEmail);
        };

        self.verify_password(password, account.password_hash.into_inner())
            .await?;

        Ok(account.account_id)
    }
}
