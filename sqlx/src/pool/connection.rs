use super::shared::{DecrementSizeGuard, SharedPool};
use crate::{Connection, Runtime};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

/// A connection managed by a [`Pool`][crate::pool::Pool].
///
/// Will be returned to the pool on-drop.
pub struct Pooled<Rt: Runtime, C: Connection<Rt>> {
    live: Option<C>,
    pub(crate) pool: Arc<SharedPool<Rt, C>>,
}

pub(super) struct Live<Rt: Runtime, C: Connection<Rt>> {
    pub(super) raw: C,
    pub(super) created: Instant,
    _rt: PhantomData<Rt>,
}

pub(super) struct Idle<Rt: Runtime, C: Connection<Rt>> {
    pub(super) live: Live<Rt, C>,
    pub(super) since: Instant,
}

/// RAII wrapper for connections being handled by functions that may drop them
pub(super) struct Floating<'pool, C> {
    inner: C,
    guard: DecrementSizeGuard<'pool>,
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<Rt: Runtime, C: Connection<Rt>> Debug for Pooled<Rt, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Show the type name of the connection ?
        f.debug_struct("PoolConnection").finish()
    }
}

impl<Rt: Runtime, C: Connection<Rt>> Deref for Pooled<Rt, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.live.as_ref().expect(DEREF_ERR).raw
    }
}

impl<Rt: Runtime, C: Connection<Rt>> DerefMut for Pooled<Rt, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live.as_mut().expect(DEREF_ERR).raw
    }
}

impl<Rt: Runtime, C: Connection<Rt>> Pooled<Rt, C> {
    /// Explicitly release a connection from the pool
    pub fn release(mut self) -> C {
        self.live.take().expect("PoolConnection double-dropped").float(&self.pool).detach()
    }
}

/// Returns the connection to the [`Pool`][crate::pool::Pool] it was checked-out from.
impl<Rt: Runtime, C: Connection<Rt>> Drop for Pooled<Rt, C> {
    fn drop(&mut self) {
        if let Some(live) = self.live.take() {
            self.pool.release(live);
        }
    }
}

impl<Rt: Runtime, C: Connection<Rt>> Live<Rt, C> {
    pub fn float(self, guard: DecrementSizeGuard<'_>) -> Floating<'_, Self> {
        Floating { inner: self, guard }
    }

    pub fn into_idle(self) -> Idle<Rt, C> {
        Idle { live: self, since: Instant::now() }
    }
}

impl<Rt: Runtime, C: Connection<Rt>> Idle<Rt, C> {
    pub fn float(self, guard: DecrementSizeGuard<'_>) -> Floating<'_, Self> {
        Floating { inner: self, guard }
    }
}

impl<Rt: Runtime, C: Connection<Rt>> Deref for Idle<Rt, C> {
    type Target = Live<Rt, C>;

    fn deref(&self) -> &Self::Target {
        &self.live
    }
}

impl<Rt: Runtime, C: Connection<Rt>> DerefMut for Idle<Rt, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live
    }
}

impl<'s, C> Floating<'s, C> {
    pub fn into_leakable(self) -> C {
        self.guard.cancel();
        self.inner
    }

    pub fn same_pool(&self, other: &SharedPool<Rt, C>) -> bool {
        self.guard.same_pool(other)
    }
}

impl<'s, Rt: Runtime, C: Connection<C>> Floating<'s, Live<Rt, C>> {
    pub fn attach(self, pool: &Arc<SharedPool<Rt, C>>) -> Pooled<Rt, C> {
        let Floating { inner, guard } = self;

        debug_assert!(guard.same_pool(pool), "BUG: attaching connection to different pool");

        guard.cancel();
        Pooled { live: Some(inner), pool: Arc::clone(pool) }
    }

    pub fn detach(self) -> C {
        self.inner.raw
    }

    pub fn into_idle(self) -> Floating<'s, Idle<Rt, C>> {
        Floating { inner: self.inner.into_idle(), guard: self.guard }
    }
}

impl<'s, Rt: Runtime, C: Connection<Rt>> Floating<'s, Idle<Rt, C>> {
    pub fn into_live(self) -> Floating<'s, Live<Rt, C>> {
        Floating { inner: self.inner.live, guard: self.guard }
    }

    pub async fn close(self) -> crate::Result<()> {
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
