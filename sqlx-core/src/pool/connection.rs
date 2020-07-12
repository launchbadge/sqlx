use super::inner::{DecrementSizeGuard, SharedPool};
use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use sqlx_rt::spawn;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

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
pub(super) struct Floating<'p, C> {
    inner: C,
    guard: DecrementSizeGuard<'p>,
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

impl<DB: Database> PoolConnection<DB> {
    // explicitly release a connection from the pool
    pub fn release(mut self) -> DB::Connection {
        self.live.take().expect("PoolConnection double-dropped").raw
    }
}

/// Returns the connection to the [`Pool`][crate::pool::Pool] it was checked-out from.
impl<DB: Database> Drop for PoolConnection<DB> {
    fn drop(&mut self) {
        if let Some(mut live) = self.live.take() {
            let pool = self.pool.clone();

            if live.raw.should_flush() {
                spawn(async move {
                    // flush the connection (will immediately return if not needed) before
                    // we fully release to the pool
                    if let Err(e) = live.raw.flush().await {
                        log::error!("error occurred while flushing the connection: {}", e);

                        // we now consider the connection to be broken
                        // close the connection and drop from the pool
                        let _ = live.float(&pool).into_idle().close().await;
                    } else {
                        // after we have flushed successfully, release to the pool
                        pool.release(live.float(&pool));
                    }
                });
            } else {
                // nothing to flush, release immediately outside of a spawn
                pool.release(live.float(&pool));
            }
        }
    }
}

impl<DB: Database> Live<DB> {
    pub fn float(self, pool: &SharedPool<DB>) -> Floating<'_, Self> {
        Floating {
            inner: self,
            guard: DecrementSizeGuard::new(pool),
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

impl<'s, C> Floating<'s, C> {
    pub fn into_leakable(self) -> C {
        self.guard.cancel();
        self.inner
    }
}

impl<'s, DB: Database> Floating<'s, Live<DB>> {
    pub fn new_live(conn: DB::Connection, guard: DecrementSizeGuard<'s>) -> Self {
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

    pub fn into_idle(self) -> Floating<'s, Idle<DB>> {
        Floating {
            inner: self.inner.into_idle(),
            guard: self.guard,
        }
    }
}

impl<'s, DB: Database> Floating<'s, Idle<DB>> {
    pub fn from_idle(idle: Idle<DB>, pool: &'s SharedPool<DB>) -> Self {
        Self {
            inner: idle,
            guard: DecrementSizeGuard::new(pool),
        }
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        self.live.raw.ping().await
    }

    pub fn into_live(self) -> Floating<'s, Live<DB>> {
        Floating {
            inner: self.inner.live,
            guard: self.guard,
        }
    }

    pub async fn close(self) -> Result<(), Error> {
        // `guard` is dropped as intended
        self.inner.live.raw.close().await
    }
}

impl<C> Deref for Floating<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> DerefMut for Floating<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
