use crate::{postgres::Connection as C, ConnectOptions};
use crossbeam_queue::{ArrayQueue, SegQueue};
use futures::TryFutureExt;
use futures::channel::oneshot;
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

// TODO: Reap old connections
// TODO: Clean up (a lot) and document what's going on
// TODO: sqlx::ConnectOptions needs to be removed and replaced with URIs everywhere
// TODO: Make Pool generic over Backend (requires a generic sqlx::Connection type)

#[derive(Clone)]
pub struct Pool {
    inner: Arc<InnerPool>,
}

struct InnerPool {
    options: ConnectOptions<'static>,
    idle: ArrayQueue<Idle>,
    waiters: SegQueue<oneshot::Sender<Live>>,
    total: AtomicUsize,
}

impl Pool {
    pub fn new<'a>(options: ConnectOptions<'a>) -> Pool {
        Pool {
            inner: Arc::new(InnerPool {
                options: options.into_owned(),
                idle: ArrayQueue::new(10),
                total: AtomicUsize::new(0),
                waiters: SegQueue::new()
            }),
        }
    }

    pub async fn acquire(&self) -> io::Result<Connection> {
        self.inner
            .acquire()
            .map_ok(|live| Connection::new(live, &self.inner))
            .await
    }
}

impl InnerPool {
    async fn acquire(&self) -> io::Result<Live> {
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

        let connection = C::establish(self.options.clone()).await?;
        let live = Live {
            connection,
            since: Instant::now(),
        };

        Ok(live)
    }

    fn release(&self, mut connection: Live) {
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
pub struct Connection {
    connection: Option<Live>,
    pool: Arc<InnerPool>,
}

impl Connection {
    fn new(connection: Live, pool: &Arc<InnerPool>) -> Self {
        Self {
            connection: Some(connection),
            pool: Arc::clone(pool),
        }
    }
}

impl Deref for Connection {
    type Target = C;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // PANIC: Will not panic unless accessed after drop
        &self.connection.as_ref().unwrap().connection
    }
}

impl DerefMut for Connection {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // PANIC: Will not panic unless accessed after drop
        &mut self.connection.as_mut().unwrap().connection
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        log::debug!("release: dropping connection; store back in queue");
        if let Some(connection) = self.connection.take() {
            self.pool.release(connection);
        }
    }
}

struct Idle {
    connection: Live,
    since: Instant,
}

struct Live {
    connection: C,
    since: Instant,
}
