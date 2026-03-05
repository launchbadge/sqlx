use std::ops::{Deref, DerefMut};

use crate::error::Error;
use crate::query_scalar::query_scalar;
use crate::Either;
use crate::MssqlConnection;

/// The lock mode for a MSSQL advisory lock.
///
/// Maps to the `@LockMode` parameter of `sp_getapplock`.
#[derive(Debug, Clone, Copy, Default)]
pub enum MssqlAdvisoryLockMode {
    /// A shared lock, compatible with other `Shared` and `Update` locks.
    Shared,

    /// An update lock, compatible with `Shared` but not with other `Update` or `Exclusive`.
    Update,

    /// An exclusive lock, incompatible with all other lock modes.
    #[default]
    Exclusive,
}

impl MssqlAdvisoryLockMode {
    fn as_str(&self) -> &'static str {
        match self {
            MssqlAdvisoryLockMode::Shared => "Shared",
            MssqlAdvisoryLockMode::Update => "Update",
            MssqlAdvisoryLockMode::Exclusive => "Exclusive",
        }
    }
}

/// A session-scoped advisory lock backed by SQL Server's `sp_getapplock` /
/// `sp_releaseapplock`.
///
/// Advisory locks are cooperative: they don't block access to any database
/// object; instead, all participants must explicitly acquire the same named
/// lock. The lock is scoped to the database session (connection).
///
/// # RAII Guard
///
/// Use [`acquire_guard`][Self::acquire_guard] or
/// [`try_acquire_guard`][Self::try_acquire_guard] to get an
/// [`MssqlAdvisoryLockGuard`] that provides access to the underlying connection
/// and can release the lock via [`release_now()`][MssqlAdvisoryLockGuard::release_now].
///
/// Unlike PostgreSQL, MSSQL connections cannot queue commands for deferred
/// execution, so the lock **cannot** be released automatically on drop.
/// If the guard is dropped without calling `release_now()` or `leak()`, a
/// warning is logged. The lock will still be released when the connection
/// is closed or returned to the pool.
///
/// For manual lock management without a guard, use [`acquire`][Self::acquire],
/// [`try_acquire`][Self::try_acquire], and [`release`][Self::release].
///
/// # Resource Name
///
/// SQL Server limits resource names to 255 characters. The name is passed as a
/// query parameter, so SQL injection is not possible.
///
/// # Example
///
/// ```rust,no_run
/// # async fn example(conn: &mut sqlx::mssql::MssqlConnection) -> sqlx::Result<()> {
/// use sqlx::mssql::MssqlAdvisoryLock;
///
/// let lock = MssqlAdvisoryLock::new("my_app_lock");
///
/// // Using the RAII guard (preferred):
/// let guard = lock.acquire_guard(&mut *conn).await?;
/// // ... do work under the lock, using `&mut *guard` as a connection ...
/// guard.release_now().await?;
///
/// // Or manual management:
/// lock.acquire(&mut *conn).await?;
/// // ... do work ...
/// lock.release(conn).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MssqlAdvisoryLock {
    resource: String,
    mode: MssqlAdvisoryLockMode,
}

/// A wrapper for a connection that represents a held MSSQL advisory lock.
///
/// Can be acquired by [`MssqlAdvisoryLock::acquire_guard()`] or
/// [`MssqlAdvisoryLock::try_acquire_guard()`].
///
/// ### Note: Release is NOT automatic on drop!
///
/// Unlike PostgreSQL, MSSQL connections cannot queue commands for deferred
/// execution. If this guard is dropped without calling
/// [`release_now()`][Self::release_now], a warning is logged and the lock
/// remains held until the connection is closed or returned to the pool.
///
/// Always prefer calling `.release_now().await` when you are done with the lock.
pub struct MssqlAdvisoryLockGuard<C: AsMut<MssqlConnection>> {
    lock: MssqlAdvisoryLock,
    conn: Option<C>,
}

impl MssqlAdvisoryLock {
    /// Create a new advisory lock with the given resource name and the default
    /// [`Exclusive`][MssqlAdvisoryLockMode::Exclusive] mode.
    pub fn new(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            mode: MssqlAdvisoryLockMode::default(),
        }
    }

    /// Create a new advisory lock with the given resource name and lock mode.
    pub fn with_mode(resource: impl Into<String>, mode: MssqlAdvisoryLockMode) -> Self {
        Self {
            resource: resource.into(),
            mode,
        }
    }

    /// Returns the resource name of this lock.
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// Returns the lock mode.
    pub fn mode(&self) -> &MssqlAdvisoryLockMode {
        &self.mode
    }

    /// Acquire the lock, waiting indefinitely until it is available.
    ///
    /// # Errors
    ///
    /// Returns an error if `sp_getapplock` returns a negative status code
    /// (e.g. lock request was cancelled or a deadlock was detected).
    pub async fn acquire(&self, conn: &mut MssqlConnection) -> Result<(), Error> {
        let status: i32 = query_scalar(
            "DECLARE @r INT; \
             EXEC @r = sp_getapplock @Resource = @p1, @LockMode = @p2, \
             @LockOwner = 'Session', @LockTimeout = -1; \
             SELECT @r;",
        )
        .bind(&self.resource)
        .bind(self.mode.as_str())
        .fetch_one(&mut *conn)
        .await?;

        if status < 0 {
            return Err(Error::Protocol(format!(
                "sp_getapplock failed for resource '{}': status {status}{}",
                self.resource,
                applock_error_message(status),
            )));
        }

        Ok(())
    }

    /// Try to acquire the lock without waiting.
    ///
    /// Returns `Ok(true)` if the lock was acquired, `Ok(false)` if it was not
    /// available (timeout).
    pub async fn try_acquire(&self, conn: &mut MssqlConnection) -> Result<bool, Error> {
        let status: i32 = query_scalar(
            "DECLARE @r INT; \
             EXEC @r = sp_getapplock @Resource = @p1, @LockMode = @p2, \
             @LockOwner = 'Session', @LockTimeout = 0; \
             SELECT @r;",
        )
        .bind(&self.resource)
        .bind(self.mode.as_str())
        .fetch_one(&mut *conn)
        .await?;

        if status >= 0 {
            // 0 = granted synchronously, 1 = granted after wait
            Ok(true)
        } else if status == -1 {
            // -1 = timed out
            Ok(false)
        } else {
            Err(Error::Protocol(format!(
                "sp_getapplock failed for resource '{}': status {status}{}",
                self.resource,
                applock_error_message(status),
            )))
        }
    }

    /// Release the lock.
    ///
    /// Returns `Ok(true)` if the lock was successfully released, `Ok(false)`
    /// if the lock was not held by this session.
    pub async fn release(&self, conn: &mut MssqlConnection) -> Result<bool, Error> {
        let sql = "DECLARE @r INT; \
                   EXEC @r = sp_releaseapplock @Resource = @p1, @LockOwner = 'Session'; \
                   SELECT @r;";

        let status: i32 = query_scalar(sql)
            .bind(&self.resource)
            .fetch_one(&mut *conn)
            .await?;

        match status {
            0 => Ok(true),
            -999 => Ok(false),
            _ => Err(Error::Protocol(format!(
                "sp_releaseapplock failed for resource '{}': status {status}",
                self.resource,
            ))),
        }
    }

    /// Acquire the lock and return an RAII guard that provides access to the
    /// underlying connection.
    ///
    /// The guard does **not** release the lock on drop (see
    /// [`MssqlAdvisoryLockGuard`] for details). Call
    /// [`release_now()`][MssqlAdvisoryLockGuard::release_now] to release the
    /// lock and recover the connection.
    ///
    /// A connection-like type is required to execute the call. Allowed types
    /// include `MssqlConnection`, `PoolConnection<Mssql>`, and mutable
    /// references to either.
    pub async fn acquire_guard<C: AsMut<MssqlConnection>>(
        &self,
        mut conn: C,
    ) -> Result<MssqlAdvisoryLockGuard<C>, Error> {
        self.acquire(conn.as_mut()).await?;
        Ok(MssqlAdvisoryLockGuard::new(self.clone(), conn))
    }

    /// Try to acquire the lock without waiting, returning an RAII guard on
    /// success.
    ///
    /// Returns `Ok(Left(guard))` if the lock was acquired, or
    /// `Ok(Right(conn))` if it was not available.
    pub async fn try_acquire_guard<C: AsMut<MssqlConnection>>(
        &self,
        mut conn: C,
    ) -> Result<Either<MssqlAdvisoryLockGuard<C>, C>, Error> {
        if self.try_acquire(conn.as_mut()).await? {
            Ok(Either::Left(MssqlAdvisoryLockGuard::new(
                self.clone(),
                conn,
            )))
        } else {
            Ok(Either::Right(conn))
        }
    }

    /// Execute `sp_releaseapplock` for this lock's resource on the given
    /// connection.
    ///
    /// This is provided for manually releasing the lock from connections
    /// returned by [`MssqlAdvisoryLockGuard::leak()`].
    ///
    /// Returns `Ok((conn, true))` if released, `Ok((conn, false))` if the lock
    /// was not held.
    pub async fn force_release<C: AsMut<MssqlConnection>>(
        &self,
        mut conn: C,
    ) -> Result<(C, bool), Error> {
        let released = self.release(conn.as_mut()).await?;
        Ok((conn, released))
    }
}

const NONE_ERR: &str = "BUG: MssqlAdvisoryLockGuard.conn taken";

impl<C: AsMut<MssqlConnection>> MssqlAdvisoryLockGuard<C> {
    fn new(lock: MssqlAdvisoryLock, conn: C) -> Self {
        MssqlAdvisoryLockGuard {
            lock,
            conn: Some(conn),
        }
    }

    /// Release the advisory lock immediately and return the connection.
    ///
    /// This is the preferred way to release the lock. An error should only be
    /// returned if there is something wrong with the connection, in which case
    /// the lock will be automatically released when the connection is closed.
    pub async fn release_now(mut self) -> Result<C, Error> {
        let (conn, released) = self
            .lock
            .force_release(self.conn.take().expect(NONE_ERR))
            .await?;

        if !released {
            tracing::warn!(
                resource = %self.lock.resource(),
                "MssqlAdvisoryLockGuard: advisory lock was not held by the contained connection",
            );
        }

        Ok(conn)
    }

    /// Cancel the release of the advisory lock, keeping it held until the
    /// connection is closed.
    ///
    /// To manually release the lock later, see
    /// [`MssqlAdvisoryLock::force_release()`].
    pub fn leak(mut self) -> C {
        self.conn.take().expect(NONE_ERR)
    }
}

impl<C: AsMut<MssqlConnection> + AsRef<MssqlConnection>> Deref for MssqlAdvisoryLockGuard<C> {
    type Target = MssqlConnection;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().expect(NONE_ERR).as_ref()
    }
}

impl<C: AsMut<MssqlConnection> + AsRef<MssqlConnection>> DerefMut for MssqlAdvisoryLockGuard<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().expect(NONE_ERR).as_mut()
    }
}

impl<C: AsMut<MssqlConnection>> AsRef<MssqlConnection> for MssqlAdvisoryLockGuard<C>
where
    C: AsRef<MssqlConnection>,
{
    fn as_ref(&self) -> &MssqlConnection {
        self.conn.as_ref().expect(NONE_ERR).as_ref()
    }
}

impl<C: AsMut<MssqlConnection>> AsMut<MssqlConnection> for MssqlAdvisoryLockGuard<C> {
    fn as_mut(&mut self) -> &mut MssqlConnection {
        self.conn.as_mut().expect(NONE_ERR).as_mut()
    }
}

/// Logs a warning if dropped without calling `release_now()` or `leak()`.
///
/// The lock remains held until the connection is closed or returned to the pool.
impl<C: AsMut<MssqlConnection>> Drop for MssqlAdvisoryLockGuard<C> {
    fn drop(&mut self) {
        if self.conn.is_some() {
            tracing::warn!(
                resource = %self.lock.resource(),
                "MssqlAdvisoryLockGuard dropped without calling release_now() or leak(). \
                 The lock will be released when the connection is closed.",
            );
        }
    }
}

fn applock_error_message(status: i32) -> &'static str {
    match status {
        -1 => " (timed out)",
        -2 => " (lock request cancelled)",
        -3 => " (deadlock victim)",
        -999 => " (parameter validation or other call error)",
        _ => "",
    }
}
