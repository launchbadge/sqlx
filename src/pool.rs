use crate::{backend::Backend, Connection};
use crossbeam_queue::{ArrayQueue, SegQueue};
use futures::{channel::oneshot, TryFutureExt};
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use url::Url;

// TODO: Reap old connections
// TODO: Clean up (a lot) and document what's going on

pub struct PoolOptions {
    pub max_size: usize,
    pub min_idle: Option<usize>,
    pub max_lifetime: Option<Duration>,
    pub idle_timeout: Option<Duration>,
    pub connection_timeout: Option<Duration>,
}

/// A database connection pool.
pub struct Pool<Conn: Connection> {
    inner: Arc<InnerPool<Conn>>,
}

struct InnerPool<Conn: Connection> {
    url: String,
    idle: ArrayQueue<Idle<Conn>>,
    waiters: SegQueue<oneshot::Sender<Live<Conn>>>,
    // Total count of connections managed by this connection pool
    total: AtomicUsize,
    options: PoolOptions,
}

pub struct PoolConnection<Conn: Connection> {
    connection: Option<Live<Conn>>,
    pool: Arc<InnerPool<Conn>>,
}

impl<Conn: Connection> Clone for Pool<Conn> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Conn: Connection> Pool<Conn> {
    pub fn new<'a>(url: &str, options: PoolOptions) -> Self {
        Self {
            inner: Arc::new(InnerPool {
                url: url.to_owned(),
                idle: ArrayQueue::new(options.max_size),
                total: AtomicUsize::new(0),
                waiters: SegQueue::new(),
                options,
            }),
        }
    }

    pub async fn acquire(&self) -> io::Result<PoolConnection<Conn>> {
        self.inner
            .acquire()
            .map_ok(|live| PoolConnection::new(live, &self.inner))
            .await
    }
}

impl<Conn: Connection> InnerPool<Conn> {
    async fn acquire(&self) -> io::Result<Live<Conn>> {
        if let Ok(idle) = self.idle.pop() {
            log::debug!("acquire: found idle connection");

            return Ok(idle.connection);
        }

        let total = self.total.load(Ordering::SeqCst);

        if total >= self.options.max_size {
            // Too many already, add a waiter and wait for
            // a free connection
            log::debug!("acquire: too many open connections; waiting for a free connection");

            let (sender, reciever) = oneshot::channel();

            self.waiters.push(sender);

            // TODO: Handle errors here
            let live = reciever.await.unwrap();

            log::debug!("acquire: free connection now available");

            return Ok(live);
        }

        self.total.store(total + 1, Ordering::SeqCst);
        log::debug!("acquire: no idle connections; establish new connection");

        let connection = Conn::establish(&self.url).await?;

        let live = Live {
            connection,
            since: Instant::now(),
        };

        Ok(live)
    }

    fn release(&self, mut connection: Live<Conn>) {
        while let Ok(waiter) = self.waiters.pop() {
            connection = match waiter.send(connection) {
                Ok(()) => {
                    return;
                }

                Err(connection) => connection,
            };
        }

        let _ = self.idle.push(Idle {
            connection,
            since: Instant::now(),
        });
    }
}

impl<Conn: Connection> PoolConnection<Conn> {
    fn new(connection: Live<Conn>, pool: &Arc<InnerPool<Conn>>) -> Self {
        Self {
            connection: Some(connection),
            pool: Arc::clone(pool),
        }
    }
}

impl<Conn: Connection> Deref for PoolConnection<Conn> {
    type Target = Conn;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // PANIC: Will not panic unless accessed after drop
        &self.connection.as_ref().unwrap().connection
    }
}

impl<Conn: Connection> DerefMut for PoolConnection<Conn> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // PANIC: Will not panic unless accessed after drop
        &mut self.connection.as_mut().unwrap().connection
    }
}

impl<Conn: Connection> Drop for PoolConnection<Conn> {
    fn drop(&mut self) {
        log::debug!("release: dropping connection; store back in queue");
        if let Some(connection) = self.connection.take() {
            self.pool.release(connection);
        }
    }
}

struct Idle<Conn: Connection> {
    connection: Live<Conn>,
    since: Instant,
}

struct Live<Conn: Connection> {
    connection: Conn,
    since: Instant,
}
