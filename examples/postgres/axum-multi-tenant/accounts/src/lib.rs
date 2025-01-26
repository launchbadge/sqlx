use std::error::Error;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use password_hash::PasswordHashString;

use sqlx::{PgConnection, PgTransaction};
use sqlx::types::Text;

use uuid::Uuid;

use tokio::sync::Semaphore;

#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct AccountId(pub Uuid);


pub struct AccountsManager {
    hashing_semaphore: Semaphore,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("email in-use")]
    EmailInUse,
    General(#[source]
            #[from] GeneralError),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticateError {
    #[error("unknown email")]
    UnknownEmail,
    #[error("invalid password")]
    InvalidPassword,
    General(#[source]
            #[from] GeneralError),
}

#[derive(Debug, thiserror::Error)]
pub enum GeneralError {
    Sqlx(#[source]
         #[from] sqlx::Error),
    PasswordHash(#[source] #[from] argon2::password_hash::Error),
    Task(#[source]
         #[from] tokio::task::JoinError),
}

impl AccountsManager {
    pub async fn new(conn: &mut PgConnection, max_hashing_threads: usize) -> Result<Self, GeneralError> {
        sqlx::migrate!().run(conn).await?;

        AccountsManager {
            hashing_semaphore: Semaphore::new(max_hashing_threads)
        }
    }

    async fn hash_password(&self, password: String) -> Result<PasswordHash, GeneralError> {
        let guard = self.hashing_semaphore.acquire().await
            .expect("BUG: this semaphore should not be closed");

        // We transfer ownership to the blocking task and back to ensure Tokio doesn't spawn
        // excess threads.
        let (_guard, res) = tokio::task::spawn_blocking(move || {
            let salt = argon2::password_hash::SaltString::generate(rand::thread_rng());
            (guard, Argon2::default().hash_password(password.as_bytes(), &salt))
        })
            .await?;

        Ok(res?)
    }

    async fn verify_password(&self, password: String, hash: PasswordHashString) -> Result<(), AuthenticateError> {
        let guard = self.hashing_semaphore.acquire().await
            .expect("BUG: this semaphore should not be closed");

        let (_guard, res) = tokio::task::spawn_blocking(move || {
            (guard, Argon2::default().verify_password(password.as_bytes(), &hash.password_hash()))
        }).await.map_err(GeneralError::from)?;

        if let Err(password_hash::Error::Password) = res {
            return Err(AuthenticateError::InvalidPassword);
        }

        res.map_err(GeneralError::from)?;

        Ok(())
    }

    pub async fn create(&self, txn: &mut PgTransaction, email: &str, password: String) -> Result<AccountId, CreateError> {
        // Hash password whether the account exists or not to make it harder
        // to tell the difference in the timing.
        let hash = self.hash_password(password).await?;

        // language=PostgreSQL
        sqlx::query!(
            "insert into accounts.account(email, password_hash) \
             values ($1, $2) \
             returning account_id",
            email,
            Text(hash) as Text<PasswordHash<'static>>,
        )
            .fetch_one(&mut *txn)
            .await
            .map_err(|e| if e.constraint() == Some("account_account_id_key") {
                CreateError::EmailInUse
            } else {
                GeneralError::from(e).into()
            })
    }

    pub async fn authenticate(&self, conn: &mut PgConnection, email: &str, password: String) -> Result<AccountId, AuthenticateError> {
        let maybe_account = sqlx::query!(
            "select account_id, password_hash as \"password_hash: Text<PasswordHashString>\" \
             from accounts.account \
             where email_id = $1",
            email
        )
            .fetch_optional(&mut *conn)
            .await
            .map_err(GeneralError::from)?;

        let Some(account) = maybe_account else {
            // Hash the password whether the account exists or not to hide the difference in timing.
            self.hash_password(password).await.map_err(GeneralError::from)?;
            return Err(AuthenticateError::UnknownEmail);
        };

        self.verify_password(password, account.password_hash.into())?;

        Ok(account.account_id)
    }
}
