use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::{cmp, mem, ptr};

use crossbeam_queue::ArrayQueue;

use crate::pool::connection::{Floating, Idle, Pooled};
use crate::pool::options::PoolOptions;
use crate::pool::wait_list::WaitList;
use crate::{Acquire, Connect, Connection, Runtime};

pub struct SharedPool<Rt: Runtime, C: Connection<Rt>> {
    idle: ArrayQueue<Idle<Rt, C>>,
    wait_list: WaitList,
    size: AtomicU32,
    is_closed: AtomicBool,
    pub(crate) pool_options: PoolOptions<Rt, C>,
    connect_options: <C as Connect<Rt>>::Options,
}

/// RAII guard returned by `Pool::try_increment_size()` and others.
///
/// Will decrement the pool size if dropped, to avoid semantically "leaking" connections
/// (where the pool thinks it has more connections than it does).
pub struct DecrementSizeGuard<'pool> {
    size: &'pool AtomicU32,
    wait_list: &'pool WaitList,
    dropped: bool,
}

// NOTE: neither of these may be `Copy` or `Clone`!
pub struct ConnectPermit<'pool>(DecrementSizeGuard<'pool>);
pub struct AcquirePermit<'pool>(&'pool AtomicU32); // just need a pointer to compare for sanity check

/// Returned by `SharedPool::try_acquire()`.
///
/// Compared to SQLx <= 0.5, the process of acquiring a connection is broken into distinct steps
/// in order to facilitate both blocking and nonblocking versions.
pub enum TryAcquireResult<'pool, Rt: Runtime, C: Connection<Rt>> {
    /// A connection has been acquired from the idle queue.
    ///
    /// Depending on the pool settings, it may still need to be tested for liveness before being
    /// returned to the user.
    Acquired(Floating<'pool, Idle<Rt, C>>),
    /// The pool's current size dropped below its maximum and a new connection may be opened.
    ///
    /// Call `.connect_async()` or `.connect_blocking()` with the given permit.
    Connect(ConnectPermit<'pool>),
    /// The task or thread should wait and call `.try_acquire()` again.
    ///
    /// The inner value is the same `AcquirePermit` that was passed to `.try_acquire()`.
    Wait,
    /// The pool is closed; the attempt to acquire the connection should return an error.
    PoolClosed,
}

impl<Rt: Runtime, C: Connection<Rt>> SharedPool<Rt, C> {
    pub fn new(
        pool_options: PoolOptions<Rt, C>,
        connect_options: <C as Connect<Rt>>::Options,
    ) -> Self {
        Self {
            idle: ArrayQueue::new(pool_options.max_connections as usize),
            wait_list: WaitList::new(),
            size: AtomicU32::new(0),
            is_closed: AtomicBool::new(false),
            pool_options,
            connect_options,
        }
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    /// Attempt to acquire a connection.
    ///
    /// If `permit` is `Some`,
    pub fn try_acquire(&self, permit: Option<AcquirePermit<'_>>) -> TryAcquireResult<'_, C> {
        use TryAcquireResult::*;

        assert!(
            permit.map_or(true, |permit| ptr::eq(&self.size, permit.0)),
            "BUG: given AcquirePermit is from a different pool"
        );

        if self.is_closed() {
            return PoolClosed;
        }

        // if the user has an `AcquirePermit`, then they've already waited at least once
        // and we should try to get them a connection immediately if possible;
        //
        // otherwise, we can immediately return a connection or `ConnectPermit` if no one is waiting
        if permit.is_some() || self.wait_list.is_empty() {
            // try to pull a connection from the idle queue
            if let Some(idle) = self.idle.pop() {
                return Acquired(idle.float(self));
            }

            // try to bump `self.size`
            if let Some(guard) = self.try_increment_size() {
                return Connect(ConnectPermit(guard));
            }
        }

        // check again after the others to make sure
        if self.is_closed() {
            return PoolClosed;
        }

        Wait
    }

    /// Attempt to increment the current size, failing if it would exceed the maximum size.
    fn try_increment_size(&self) -> Option<DecrementSizeGuard<'_>> {
        if self.is_closed() {
            return None;
        }

        self.size
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |size| {
                (size < todo!("self.options.max_connections")).then(|| size + 1)
            })
            .ok()
            .map(|_| DecrementSizeGuard::new(self))
    }
}

#[cfg(feature = "async")]
impl<Rt: crate::Async, C: Connection<Rt>> SharedPool<Rt, C> {
    pub async fn wait_async(&self) -> AcquirePermit {
        self.wait_list.wait().await;
        AcquirePermit(&self.size)
    }

    pub async fn connect_async(
        self: &Arc<Self>,
        permit: ConnectPermit,
    ) -> crate::Result<Pooled<Rt, C>>
    where
        C: crate::Connect<Rt>,
    {
        assert!(permit.0.same_pool(self), "BUG: ConnectPermit is from a different pool!");

        let mut conn = crate::Connect::connect_with(&self.connect_options)
            .await
            .map(|c| Floating::new_live(c, permit.0))?;

        if let Some(ref after_connect) = self.pool_options.after_connect_async {
            after_connect(&mut conn).await?;
        }

        Ok(conn.attach(self))
    }

    pub async fn on_acquire_async(
        self: &Arc<Self>,
        conn: &mut Floating<'_, C>,
    ) -> crate::Result<()> {
        assert!(conn.same_pool(self), "BUG: connection is from a different pool");

        if let Some(ref before_acquire) = self.pool_options.before_acquire_async {
            before_acquire(conn).await?;
        }

        Ok(())
    }

    pub async fn init_min_connections_async<Rt: Runtime, C: Connection<Rt>>(
        &mut self,
    ) -> crate::Result<()> {
        for _ in 0..cmp::max(self.pool_options.min_connections, 1) {
            // this guard will prevent us from exceeding `max_size`
            if let Some(guard) = self.try_increment_size() {
                // [connect] will raise an error when past deadline
                let conn = self.connect_async(ConnectPermit(guard)).await?;
                let is_ok = self.idle.push(conn.into_idle().into_leakable()).is_ok();

                if !is_ok {
                    panic!("BUG: connection queue overflow in init_min_connections");
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl<C: Connection<crate::Blocking>> SharedPool<crate::Blocking, C> {
    pub fn wait_blocking(&self, deadline: Option<Instant>) -> Option<AcquirePermit<'_>> {
        self.wait_list.wait().block_on(deadline).then(|| AcquirePermit(&self.size))
    }

    pub fn connect_blocking(
        self: &Arc<Self>,
        permit: ConnectPermit<'_>,
    ) -> crate::Result<Pooled<crate::Blocking, C>>
    where
        C: crate::blocking::Connect<crate::Blocking>,
    {
        assert!(permit.0.same_pool(self), "BUG: ConnectPermit is from a different pool!");

        crate::blocking::Connect::connect_with(&self.connect_options)
            .map(|c| Floating::new_live(c, permit.0).attach(self))
    }

    pub fn on_acquire_blocking(self: &Arc<Self>, conn: &mut Floating<'_, C>) -> crate::Result<()> {
        assert!(conn.same_pool(self), "BUG: connection is from a different pool");

        if let Some(ref before_acquire) = self.pool_options.before_acquire_blocking {
            before_acquire(conn)?;
        }

        Ok(())
    }

    pub fn init_min_connections_blocking<Rt: Runtime, C: Connection<Rt>>(
        &mut self,
    ) -> crate::Result<()> {
        for _ in 0..cmp::max(self.pool_options.min_connections, 1) {
            // this guard will prevent us from exceeding `max_size`
            if let Some(guard) = self.try_increment_size() {
                // [connect] will raise an error when past deadline
                let conn = self.connect_blocking(ConnectPermit(guard))?;
                let is_ok = self.idle.push(conn.into_idle().into_leakable()).is_ok();

                if !is_ok {
                    panic!("BUG: connection queue overflow in init_min_connections");
                }
            }
        }

        Ok(())
    }
}

impl<'pool> DecrementSizeGuard<'pool> {
    fn new<Rt: Runtime, C: Connection<Rt>>(pool: &'pool SharedPool<Rt, C>) -> Self {
        Self { size: &pool.size, wait_list: &pool.wait_list, dropped: false }
    }

    /// Return `true` if the internal references point to the same fields in `SharedPool`.
    pub fn same_pool<Rt: Runtime, C: Connection<Rt>>(
        &self,
        pool: &'pool SharedPool<Rt, C>,
    ) -> bool {
        ptr::eq(self.size, &pool.size) && ptr::eq(self.wait_list, &pool.wait_list)
    }

    pub fn cancel(self) {
        mem::forget(self);
    }
}

impl Drop for DecrementSizeGuard<'_> {
    fn drop(&mut self) {
        assert!(!self.dropped, "double-dropped!");
        self.dropped = true;
        self.size.fetch_sub(1, Ordering::SeqCst);
        self.wait_list.wake_one();
    }
}
