use crate::{
    backend::Backend,
    connection::Connection,
    describe::Describe,
    error::Error,
    executor::Executor,
    params::IntoQueryParameters,
    row::{FromRow, Row},
};
use async_std::{
    sync::{channel, Receiver, Sender},
    task,
};
use futures_channel::oneshot;
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::{future::FutureExt, stream::StreamExt};
use std::{
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

/// A pool of database connections.
pub struct Pool<DB>(Arc<SharedPool<DB>>)
where
    DB: Backend;

impl<DB> Pool<DB>
where
    DB: Backend,
{
    /// Creates a connection pool with the default configuration.
    pub async fn new(url: &str) -> crate::Result<Self> {
        Ok(Pool(Arc::new(
            SharedPool::new(url, Options::default()).await?,
        )))
    }

    /// Returns a [Builder] to configure a new connection pool.
    pub fn builder() -> Builder<DB> {
        Builder::new()
    }

    /// Retrieves a connection from the pool.
    ///
    /// Waits for at most the configured connection timeout before returning an error.
    pub async fn acquire(&self) -> crate::Result<Connection<DB>> {
        let live = self.0.acquire().await?;
        Ok(Connection::new(live, Some(Arc::clone(&self.0))))
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` if there are no idle connections available in the pool.
    /// This method will not block waiting to establish a new connection.
    pub fn try_acquire(&self) -> Option<Connection<DB>> {
        let live = self.0.try_acquire()?;
        Some(Connection::new(live, Some(Arc::clone(&self.0))))
    }

    /// Ends the use of a connection pool. Prevents any new connections
    /// and will close all active connections when they are returned to the pool.
    ///
    /// Does not resolve until all connections are closed.
    pub async fn close(&self) {
        unimplemented!()
    }

    /// Returns the number of connections currently being managed by the pool.
    pub fn size(&self) -> u32 {
        self.0.size.load(Ordering::Acquire)
    }

    /// Returns the number of idle connections.
    pub fn idle(&self) -> usize {
        self.0.pool_rx.len()
    }

    /// Returns the configured maximum pool size.
    pub fn max_size(&self) -> u32 {
        self.0.options.max_size
    }

    /// Returns the configured mimimum idle connection count.
    pub fn min_idle(&self) -> Option<u32> {
        self.0.options.min_idle
    }

    /// Returns the configured maximum connection lifetime.
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.0.options.max_lifetime
    }

    /// Returns the configured idle connection timeout.
    pub fn idle_timeout(&self) -> Option<Duration> {
        self.0.options.idle_timeout
    }
}

/// Returns a new [Pool] tied to the same shared connection pool.
impl<DB> Clone for Pool<DB>
where
    DB: Backend,
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Default)]
pub struct Builder<DB>
where
    DB: Backend,
{
    phantom: PhantomData<DB>,
    options: Options,
}

impl<DB> Builder<DB>
where
    DB: Backend,
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
            options: Options::default(),
        }
    }

    pub fn max_size(mut self, max_size: u32) -> Self {
        self.options.max_size = max_size;
        self
    }

    pub fn min_idle(mut self, min_idle: impl Into<Option<u32>>) -> Self {
        self.options.min_idle = min_idle.into();
        self
    }

    pub fn max_lifetime(mut self, max_lifetime: impl Into<Option<Duration>>) -> Self {
        self.options.max_lifetime = max_lifetime.into();
        self
    }

    pub fn idle_timeout(mut self, idle_timeout: impl Into<Option<Duration>>) -> Self {
        self.options.idle_timeout = idle_timeout.into();
        self
    }

    pub async fn build(self, url: &str) -> crate::Result<Pool<DB>> {
        Ok(Pool(Arc::new(SharedPool::new(url, self.options).await?)))
    }
}

struct Options {
    max_size: u32,
    min_idle: Option<u32>,
    max_lifetime: Option<Duration>,
    idle_timeout: Option<Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: None,
            max_lifetime: None,
            idle_timeout: None,
        }
    }
}

pub(crate) struct SharedPool<DB>
where
    DB: Backend,
{
    url: String,
    pool_rx: Receiver<Idle<DB>>,
    pool_tx: Sender<Idle<DB>>,
    size: AtomicU32,
    options: Options,
}

impl<DB> SharedPool<DB>
where
    DB: Backend,
{
    async fn new(url: &str, options: Options) -> crate::Result<Self> {
        // TODO: Establish [min_idle] connections

        let (pool_tx, pool_rx) = channel(options.max_size as usize);

        Ok(Self {
            url: url.to_owned(),
            pool_rx,
            pool_tx,
            size: AtomicU32::new(0),
            options,
        })
    }

    #[inline]
    fn try_acquire(&self) -> Option<Live<DB>> {
        Some(self.pool_rx.recv().now_or_never()??.live(&self.pool_tx))
    }

    async fn acquire(&self) -> crate::Result<Live<DB>> {
        if let Some(live) = self.try_acquire() {
            return Ok(live);
        }

        loop {
            let size = self.size.load(Ordering::Acquire);

            if size >= self.options.max_size {
                // Too many open connections
                // Wait until one is available

                // Waiters are not dropped unless the pool is dropped
                // which would drop this future
                return Ok(self
                    .pool_rx
                    .recv()
                    .await
                    .expect("waiter dropped without dropping pool")
                    .live(&self.pool_tx));
            }

            if self.size.compare_and_swap(size, size + 1, Ordering::AcqRel) == size {
                // Open a new connection and return directly
                let raw = DB::open(&self.url).await?;
                return Ok(Live::pooled(raw, &self.pool_tx));
            }
        }
    }
}

impl<DB> Executor for Pool<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'e, 'q: 'e, I: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
    {
        Box::pin(async move { <&Pool<DB> as Executor>::execute(&mut &*self, query, params).await })
    }

    fn fetch<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxStream<'e, crate::Result<T>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut self_ = &*self;
            let mut s = <&Pool<DB> as Executor>::fetch(&mut self_, query, params);

            while let Some(row) = s.next().await.transpose()? {
                yield row;
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<Option<T>>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send,
    {
        Box::pin(async move {
            <&Pool<DB> as Executor>::fetch_optional(&mut &*self, query, params).await
        })
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Backend>>> {
        Box::pin(async move { <&Pool<DB> as Executor>::describe(&mut &*self, query).await })
    }
}

impl<DB> Executor for &'_ Pool<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'e, 'q: 'e, I: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
    {
        Box::pin(async move { self.0.acquire().await?.execute(query, params).await })
    }

    fn fetch<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxStream<'e, crate::Result<T>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut live = self.0.acquire().await?;
            let mut s = live.fetch(query, params);

            while let Some(row) = s.next().await.transpose()? {
                yield row;
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<Option<T>>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send,
    {
        Box::pin(async move { self.0.acquire().await?.fetch_optional(query, params).await })
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Backend>>> {
        Box::pin(async move { self.0.acquire().await?.describe(query).await })
    }
}

struct Raw<DB> {
    pub(crate) inner: DB,
    pub(crate) created: Instant,
}

struct Idle<DB>
where
    DB: Backend,
{
    raw: Raw<DB>,
    #[allow(unused)]
    since: Instant,
}

impl<DB: Backend> Idle<DB> {
    fn live(self, pool_tx: &Sender<Idle<DB>>) -> Live<DB> {
        Live {
            raw: Some(self.raw),
            pool_tx: Some(pool_tx.clone()),
        }
    }
}

pub(crate) struct Live<DB>
where
    DB: Backend,
{
    raw: Option<Raw<DB>>,
    pool_tx: Option<Sender<Idle<DB>>>,
}

impl<DB: Backend> Live<DB> {
    pub fn unpooled(raw: DB) -> Self {
        Live {
            raw: Some(Raw {
                inner: raw,
                created: Instant::now(),
            }),
            pool_tx: None,
        }
    }

    fn pooled(raw: DB, pool_tx: &Sender<Idle<DB>>) -> Self {
        Live {
            raw: Some(Raw {
                inner: raw,
                created: Instant::now(),
            }),
            pool_tx: Some(pool_tx.clone()),
        }
    }

    pub fn release(mut self) {
        self.release_mut()
    }

    fn release_mut(&mut self) {
        // `.release_mut()` will be called twice if `.release()` is called
        if let (Some(raw), Some(pool_tx)) = (self.raw.take(), self.pool_tx.as_ref()) {
            pool_tx
                .send(Idle {
                    raw,
                    since: Instant::now(),
                })
                .now_or_never()
                .expect("(bug) connection released into a full pool")
        }
    }
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<DB: Backend> Deref for Live<DB> {
    type Target = DB;

    fn deref(&self) -> &DB {
        &self.raw.as_ref().expect(DEREF_ERR).inner
    }
}

impl<DB: Backend> DerefMut for Live<DB> {
    fn deref_mut(&mut self) -> &mut DB {
        &mut self.raw.as_mut().expect(DEREF_ERR).inner
    }
}

impl<DB: Backend> Drop for Live<DB> {
    fn drop(&mut self) {
        self.release_mut()
    }
}
