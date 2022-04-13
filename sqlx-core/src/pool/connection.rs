use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

use futures_intrusive::sync::SemaphoreReleaser;

use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;

use super::inner::{DecrementSizeGuard, SharedPool};
use std::future::Future;

/// A connection managed by a [`Pool`][crate::pool::Pool].
///
/// Will be returned to the pool on-drop.
pub struct PoolConnection<DB: Database> {
    live: Option<Live<DB>>,
    pub(crate) pool: Arc<SharedPool<DB>>,
}

pub(super) struct Live<DB: Database> {
    pub(super) raw: DB::Connection,
    pub(super) created: Instant,
}

pub(super) struct Idle<DB: Database> {
    pub(super) live: Live<DB>,
    pub(super) since: Instant,
}

/// RAII wrapper for connections being handled by functions that may drop them
pub(super) struct Floating<DB: Database, C> {
    pub(super) inner: C,
    pub(super) guard: DecrementSizeGuard<DB>,
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<DB: Database> Debug for PoolConnection<DB> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Show the type name of the connection ?
        f.debug_struct("PoolConnection").finish()
    }
}

impl<DB: Database> Deref for PoolConnection<DB> {
    type Target = DB::Connection;

    fn deref(&self) -> &Self::Target {
        &self.live.as_ref().expect(DEREF_ERR).raw
    }
}

impl<DB: Database> DerefMut for PoolConnection<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live.as_mut().expect(DEREF_ERR).raw
    }
}

impl<DB: Database> AsRef<DB::Connection> for PoolConnection<DB> {
    fn as_ref(&self) -> &DB::Connection {
        self
    }
}

impl<DB: Database> AsMut<DB::Connection> for PoolConnection<DB> {
    fn as_mut(&mut self) -> &mut DB::Connection {
        self
    }
}

impl<DB: Database> PoolConnection<DB> {
    /// Explicitly release a connection from the pool
    #[deprecated = "renamed to `.detach()` for clarity"]
    pub fn release(self) -> DB::Connection {
        self.detach()
    }

    /// Detach this connection from the pool, allowing it to open a replacement.
    ///
    /// Note that if your application uses a single shared pool, this
    /// effectively lets the application exceed the `max_connections` setting.
    ///
    /// If you want the pool to treat this connection as permanently checked-out,
    /// use [`.leak()`][Self::leak] instead.
    pub fn detach(mut self) -> DB::Connection {
        self.live
            .take()
            .expect("PoolConnection double-dropped")
            .float(self.pool.clone())
            .detach()
    }

    /// Detach this connection from the pool, treating it as permanently checked-out.
    ///
    /// This effectively will reduce the maximum capacity of the pool by 1 every time it is used.
    ///
    /// If you don't want to impact the pool's capacity, use [`.detach()`][Self::detach] instead.
    pub fn leak(mut self) -> DB::Connection {
        self.live.take().expect("PoolConnection double-dropped").raw
    }

    /// Test the connection to make sure it is still live before returning it to the pool.
    ///
    /// This effectively runs the drop handler eagerly instead of spawning a task to do it.
    pub(crate) fn return_to_pool(&mut self) -> impl Future<Output = ()> + Send + 'static {
        // float the connection in the pool before we move into the task
        // in case the returned `Future` isn't executed, like if it's spawned into a dying runtime
        // https://github.com/launchbadge/sqlx/issues/1396
        let floating = self.live.take().map(|live| live.float(self.pool.clone()));

        async move {
            let mut floating = if let Some(floating) = floating {
                floating
            } else {
                return;
            };

            // test the connection on-release to ensure it is still viable,
            // and flush anything time-sensitive like transaction rollbacks
            // if an Executor future/stream is dropped during an `.await` call, the connection
            // is likely to be left in an inconsistent state, in which case it should not be
            // returned to the pool; also of course, if it was dropped due to an error
            // this is simply a band-aid as SQLx-next (0.6) connections should be able
            // to recover from cancellations
            if let Err(e) = floating.raw.ping().await {
                log::warn!(
                    "error occurred while testing the connection on-release: {}",
                    e
                );

                // we now consider the connection to be broken; just drop it to close
                // trying to close gracefully might cause something weird to happen
                drop(floating);
            } else {
                // if the connection is still viable, release it to the pool
                floating.release();
            }
        }
    }
}

/// Returns the connection to the [`Pool`][crate::pool::Pool] it was checked-out from.
impl<DB: Database> Drop for PoolConnection<DB> {
    fn drop(&mut self) {
        if self.live.is_some() {
            #[cfg(not(feature = "_rt-async-std"))]
            if let Ok(handle) = sqlx_rt::Handle::try_current() {
                handle.spawn(self.return_to_pool());
            }

            #[cfg(feature = "_rt-async-std")]
            sqlx_rt::spawn(self.return_to_pool());
        }
    }
}

impl<DB: Database> Live<DB> {
    pub fn float(self, pool: Arc<SharedPool<DB>>) -> Floating<DB, Self> {
        Floating {
            inner: self,
            // create a new guard from a previously leaked permit
            guard: DecrementSizeGuard::new_permit(pool),
        }
    }

    pub fn into_idle(self) -> Idle<DB> {
        Idle {
            live: self,
            since: Instant::now(),
        }
    }
}

impl<DB: Database> Deref for Idle<DB> {
    type Target = Live<DB>;

    fn deref(&self) -> &Self::Target {
        &self.live
    }
}

impl<DB: Database> DerefMut for Idle<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live
    }
}

impl<DB: Database> Floating<DB, Live<DB>> {
    pub fn new_live(conn: DB::Connection, guard: DecrementSizeGuard<DB>) -> Self {
        Self {
            inner: Live {
                raw: conn,
                created: Instant::now(),
            },
            guard,
        }
    }

    pub fn attach(self, pool: &Arc<SharedPool<DB>>) -> PoolConnection<DB> {
        let Floating { inner, guard } = self;

        debug_assert!(
            guard.same_pool(pool),
            "BUG: attaching connection to different pool"
        );

        guard.cancel();
        PoolConnection {
            live: Some(inner),
            pool: Arc::clone(pool),
        }
    }

    pub fn release(self) {
        self.guard.pool.clone().release(self);
    }

    pub async fn close(self) -> Result<(), Error> {
        // `guard` is dropped as intended
        self.inner.raw.close().await
    }

    pub fn detach(self) -> DB::Connection {
        self.inner.raw
    }

    pub fn into_idle(self) -> Floating<DB, Idle<DB>> {
        Floating {
            inner: self.inner.into_idle(),
            guard: self.guard,
        }
    }
}

impl<DB: Database> Floating<DB, Idle<DB>> {
    pub fn from_idle(
        idle: Idle<DB>,
        pool: Arc<SharedPool<DB>>,
        permit: SemaphoreReleaser<'_>,
    ) -> Self {
        Self {
            inner: idle,
            guard: DecrementSizeGuard::from_permit(pool, permit),
        }
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        self.live.raw.ping().await
    }

    pub fn into_live(self) -> Floating<DB, Live<DB>> {
        Floating {
            inner: self.inner.live,
            guard: self.guard,
        }
    }

    pub async fn close(self) -> DecrementSizeGuard<DB> {
        // `guard` is dropped as intended
        if let Err(e) = self.inner.live.raw.close().await {
            log::debug!("error occurred while closing the pool connection: {}", e);
        }
        self.guard
    }
}

impl<DB: Database, C> Deref for Floating<DB, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<DB: Database, C> DerefMut for Floating<DB, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
