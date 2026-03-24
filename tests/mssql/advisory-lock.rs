use sqlx::mssql::{Mssql, MssqlAdvisoryLock, MssqlAdvisoryLockMode};
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_acquires_and_releases() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let lock = MssqlAdvisoryLock::new("sqlx_test_acquire_release");

    lock.acquire(&mut conn).await?;
    let released = lock.release(&mut conn).await?;
    assert!(released, "lock should have been held and released");

    Ok(())
}

#[sqlx_macros::test]
async fn it_try_acquire_succeeds_when_free() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let lock = MssqlAdvisoryLock::new("sqlx_test_try_free");

    let acquired = lock.try_acquire(&mut conn).await?;
    assert!(acquired, "lock should be free and acquired");

    lock.release(&mut conn).await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_try_acquire_fails_when_held() -> anyhow::Result<()> {
    let mut conn1 = new::<Mssql>().await?;
    let mut conn2 = new::<Mssql>().await?;

    let lock = MssqlAdvisoryLock::new("sqlx_test_try_held");

    // Conn1 holds the exclusive lock
    lock.acquire(&mut conn1).await?;

    // Conn2 should fail to acquire it immediately
    let acquired = lock.try_acquire(&mut conn2).await?;
    assert!(!acquired, "lock should not be available");

    // Release from conn1
    lock.release(&mut conn1).await?;

    // Now conn2 should be able to acquire
    let acquired = lock.try_acquire(&mut conn2).await?;
    assert!(acquired, "lock should now be free");

    lock.release(&mut conn2).await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_supports_shared_locks() -> anyhow::Result<()> {
    let mut conn1 = new::<Mssql>().await?;
    let mut conn2 = new::<Mssql>().await?;

    let lock = MssqlAdvisoryLock::with_mode("sqlx_test_shared", MssqlAdvisoryLockMode::Shared);

    // Both connections should be able to acquire a shared lock
    lock.acquire(&mut conn1).await?;
    let acquired = lock.try_acquire(&mut conn2).await?;
    assert!(
        acquired,
        "shared lock should be acquirable by second connection"
    );

    lock.release(&mut conn1).await?;
    lock.release(&mut conn2).await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_release_returns_false_when_not_held() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let lock = MssqlAdvisoryLock::new("sqlx_test_not_held");

    let released = lock.release(&mut conn).await?;
    assert!(
        !released,
        "release should return false when lock is not held"
    );

    Ok(())
}
