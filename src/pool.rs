use super::connection::RawConnection;
use crate::{backend::Backend, ConnectOptions};
use crossbeam_queue::{ArrayQueue, SegQueue};
use futures::{channel::oneshot, TryFutureExt};
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

// TODO: Add a sqlx::Connection type so we don't leak the RawConnection
// TODO: Reap old connections
// TODO: Clean up (a lot) and document what's going on
// TODO: sqlx::ConnectOptions needs to be removed and replaced with URIs everywhere

pub struct Pool<B>
where
    B: Backend,
{
    inner: Arc<InnerPool<B>>,
}

impl<B> Clone for Pool<B>
where
    B: Backend,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

struct InnerPool<B>
where
    B: Backend,
{
    options: ConnectOptions<'static>,
    idle: ArrayQueue<Idle<B>>,
    waiters: SegQueue<oneshot::Sender<Live<B>>>,
    total: AtomicUsize,
}

impl<B> Pool<B>
where
    B: Backend,
{
    pub fn new<'a>(options: ConnectOptions<'a>) -> Self {
        Self {
            inner: Arc::new(InnerPool {
                options: options.into_owned(),
                idle: ArrayQueue::new(10),
                total: AtomicUsize::new(0),
                waiters: SegQueue::new(),
            }),
        }
    }

    pub async fn acquire(&self) -> io::Result<Connection<B>> {
        self.inner
            .acquire()
            .map_ok(|live| Connection::new(live, &self.inner))
            .await
    }
}

impl<B> InnerPool<B>
where
    B: Backend,
{
    async fn acquire(&self) -> io::Result<Live<B>> {
        if let Ok(idle) = self.idle.pop() {
            log::debug!("acquire: found idle connection");

            return Ok(idle.connection);
        }

        let total = self.total.load(Ordering::SeqCst);

        if total >= 10 {
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

        let connection = B::RawConnection::establish(self.options.clone()).await?;
        let live = Live {
            connection,
            since: Instant::now(),
        };

        Ok(live)
    }

    fn release(&self, mut connection: Live<B>) {
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

// TODO: Need a better name here than [pool::Connection] ?

pub struct Connection<B>
where
    B: Backend,
{
    connection: Option<Live<B>>,
    pool: Arc<InnerPool<B>>,
}

impl<B> Connection<B>
where
    B: Backend,
{
    fn new(connection: Live<B>, pool: &Arc<InnerPool<B>>) -> Self {
        Self {
            connection: Some(connection),
            pool: Arc::clone(pool),
        }
    }
}

impl<B> Deref for Connection<B>
where
    B: Backend,
{
    type Target = B::RawConnection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // PANIC: Will not panic unless accessed after drop
        &self.connection.as_ref().unwrap().connection
    }
}

impl<B> DerefMut for Connection<B>
where
    B: Backend,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // PANIC: Will not panic unless accessed after drop
        &mut self.connection.as_mut().unwrap().connection
    }
}

impl<B> Drop for Connection<B>
where
    B: Backend,
{
    fn drop(&mut self) {
        log::debug!("release: dropping connection; store back in queue");
        if let Some(connection) = self.connection.take() {
            self.pool.release(connection);
        }
    }
}

struct Idle<B>
where
    B: Backend,
{
    connection: Live<B>,
    since: Instant,
}

struct Live<B>
where
    B: Backend,
{
    connection: B::RawConnection,
    since: Instant,
}
