use futures_core::future::BoxFuture;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

use super::inner::{DecrementSizeGuard, SharedPool};
use crate::connection::{Connect, Connection};

/// A connection checked out from [`Pool`][crate::Pool].
///
/// Will be returned to the pool on-drop.
pub struct PoolConnection<C>
where
    C: Connect,
{
    live: Option<Live<C>>,
    pool: Arc<SharedPool<C>>,
}

pub(super) struct Live<C> {
    raw: C,
    pub(super) created: Instant,
}

pub(super) struct Idle<C> {
    live: Live<C>,
    pub(super) since: Instant,
}

/// RAII wrapper for connections being handled by functions that may drop them
pub(super) struct Floating<'p, C> {
    inner: C,
    guard: DecrementSizeGuard<'p>,
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<C> Deref for PoolConnection<C>
where
    C: Connect,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.live.as_ref().expect(DEREF_ERR).raw
    }
}

impl<C> DerefMut for PoolConnection<C>
where
    C: Connect,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live.as_mut().expect(DEREF_ERR).raw
    }
}

impl<C> Connection for PoolConnection<C>
where
    C: Connect,
{
    /// Detach the connection from the pool and close it nicely.
    fn close(mut self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(async move {
            let live = self.live.take().expect("PoolConnection double-dropped");
            live.float(&self.pool).into_idle().close().await
        })
    }

    #[inline]
    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        Box::pin(self.deref_mut().ping())
    }
}

/// Returns the connection to the [`Pool`][crate::Pool] it was checked-out from.
impl<C> Drop for PoolConnection<C>
where
    C: Connect,
{
    fn drop(&mut self) {
        if let Some(live) = self.live.take() {
            self.pool.release(live.float(&self.pool));
        }
    }
}

impl<C> Live<C> {
    pub fn float(self, pool: &SharedPool<C>) -> Floating<Self> {
        Floating {
            inner: self,
            guard: DecrementSizeGuard::new(pool),
        }
    }

    pub fn into_idle(self) -> Idle<C> {
        Idle {
            live: self,
            since: Instant::now(),
        }
    }
}

impl<C> Deref for Idle<C> {
    type Target = Live<C>;

    fn deref(&self) -> &Self::Target {
        &self.live
    }
}

impl<C> DerefMut for Idle<C> {
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

impl<'s, C> Floating<'s, Live<C>> {
    pub fn new_live(conn: C, guard: DecrementSizeGuard<'s>) -> Self {
        Self {
            inner: Live {
                raw: conn,
                created: Instant::now(),
            },
            guard,
        }
    }

    pub fn attach(self, pool: &Arc<SharedPool<C>>) -> PoolConnection<C>
    where
        C: Connect,
    {
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

    pub fn into_idle(self) -> Floating<'s, Idle<C>> {
        Floating {
            inner: self.inner.into_idle(),
            guard: self.guard,
        }
    }
}

impl<'s, C> Floating<'s, Idle<C>> {
    pub fn from_idle(idle: Idle<C>, pool: &'s SharedPool<C>) -> Self {
        Self {
            inner: idle,
            guard: DecrementSizeGuard::new(pool),
        }
    }

    pub async fn ping(&mut self) -> crate::Result<()>
    where
        C: Connection,
    {
        self.live.raw.ping().await
    }

    pub fn into_live(self) -> Floating<'s, Live<C>> {
        Floating {
            inner: self.inner.live,
            guard: self.guard,
        }
    }

    pub async fn close(self) -> crate::Result<()>
    where
        C: Connection,
    {
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
