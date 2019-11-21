use crate::{
    backend::Backend,
    connection::{Connection, RawConnection},
    error::Error,
    executor::Executor,
    query::IntoQueryParameters,
    row::FromSqlRow,
};
use crossbeam_queue::{ArrayQueue, SegQueue};
use futures_channel::oneshot;
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::StreamExt;
use std::{
    marker::PhantomData,
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
        self.0.num_idle.load(Ordering::Acquire)
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
    idle: ArrayQueue<Idle<DB>>,
    waiters: SegQueue<oneshot::Sender<Live<DB>>>,
    size: AtomicU32,
    num_waiters: AtomicUsize,
    num_idle: AtomicUsize,
    options: Options,
}

impl<DB> SharedPool<DB>
where
    DB: Backend,
{
    async fn new(url: &str, options: Options) -> crate::Result<Self> {
        // TODO: Establish [min_idle] connections

        Ok(Self {
            url: url.to_owned(),
            idle: ArrayQueue::new(options.max_size as usize),
            waiters: SegQueue::new(),
            size: AtomicU32::new(0),
            num_idle: AtomicUsize::new(0),
            num_waiters: AtomicUsize::new(0),
            options,
        })
    }

    #[inline]
    fn try_acquire(&self) -> Option<Live<DB>> {
        if let Ok(idle) = self.idle.pop() {
            self.num_idle.fetch_sub(1, Ordering::AcqRel);

            return Some(idle.live);
        }

        None
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

                let (sender, receiver) = oneshot::channel();

                self.waiters.push(sender);
                self.num_waiters.fetch_add(1, Ordering::AcqRel);

                // Waiters are not dropped unless the pool is dropped
                // which would drop this future
                return Ok(receiver
                    .await
                    .expect("waiter dropped without dropping pool"));
            }

            if self.size.compare_and_swap(size, size + 1, Ordering::AcqRel) == size {
                // Open a new connection and return directly

                let raw = <DB as Backend>::RawConnection::establish(&self.url).await?;
                let live = Live {
                    raw,
                    since: Instant::now(),
                };

                return Ok(live);
            }
        }
    }

    pub(crate) fn release(&self, mut live: Live<DB>) {
        if self.num_waiters.load(Ordering::Acquire) > 0 {
            while let Ok(waiter) = self.waiters.pop() {
                self.num_waiters.fetch_sub(1, Ordering::AcqRel);

                live = match waiter.send(live) {
                    Ok(()) => {
                        return;
                    }

                    Err(live) => live,
                };
            }
        }

        let _ = self.idle.push(Idle {
            live,
            since: Instant::now(),
        });

        self.num_idle.fetch_add(1, Ordering::AcqRel);
    }
}

impl<DB> Executor for Pool<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'c, 'q: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
    {
        Box::pin(async move { <&Pool<DB> as Executor>::execute(&mut &*self, query, params).await })
    }

    fn fetch<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut self_ = &*self;
            let mut s = <&Pool<DB> as Executor>::fetch(&mut self_, query, params);

            while let Some(row) = s.next().await.transpose()? {
                yield row;
            }

            drop(s);
        })
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send,
    {
        Box::pin(async move {
            <&Pool<DB> as Executor>::fetch_optional(&mut &*self, query, params).await
        })
    }
}

impl<DB> Executor for &'_ Pool<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'c, 'q: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
    {
        Box::pin(async move {
            let mut live = self.0.acquire().await?;
            let result = live.raw.execute(query, params.into()).await;
            self.0.release(live);

            result
        })
    }

    fn fetch<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut live = self.0.acquire().await?;
            let mut s = live.raw.fetch(query, params.into());

            while let Some(row) = s.next().await.transpose()? {
                yield T::from_row(row);
            }

            drop(s);
            self.0.release(live);
        })
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send,
    {
        Box::pin(async move {
            let mut live = self.0.acquire().await?;
            let row = live.raw.fetch_optional(query, params.into()).await?;

            self.0.release(live);

            Ok(row.map(T::from_row))
        })
    }
}

struct Idle<DB>
where
    DB: Backend,
{
    live: Live<DB>,
    #[allow(unused)]
    since: Instant,
}

pub(crate) struct Live<DB>
where
    DB: Backend,
{
    pub(crate) raw: DB::RawConnection,
    #[allow(unused)]
    pub(crate) since: Instant,
}
