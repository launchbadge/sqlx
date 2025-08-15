use anyhow::{anyhow, Context};
use tokio::task;

use argon2::password_hash::{Salt, SaltString};
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub async fn hash(password: String) -> anyhow::Result<String> {
    task::spawn_blocking(move || {
        // `SaltString::generate()` is only compatible with `rand 0.6`, which is very out-of-date now.
        // This shows how to generate a salt using nearly any `rand` version.
        let salt: [u8; Salt::RECOMMENDED_LENGTH] = rand::random();

        let salt = SaltString::encode_b64(&salt)
            .expect("should not fail; we generated a salt of recommended length");

        Ok(Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!(e).context("failed to hash password"))?
            .to_string())
    })
    .await
    .context("panic in hash()")?
}

pub async fn verify(password: String, hash: String) -> anyhow::Result<bool> {
    task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash)
            .map_err(|e| anyhow!(e).context("BUG: password hash invalid"))?;

        let res = Argon2::default().verify_password(password.as_bytes(), &hash);

        match res {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(anyhow!(e).context("failed to verify password")),
        }
    })
    .await
    .context("panic in verify()")?
}
