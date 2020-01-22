use crossbeam_queue::{ArrayQueue, SegQueue};
use futures_channel::oneshot::{channel, Sender};

use super::conn::{Floating, Idle};
use super::size::PoolSize;
use crate::{Connection, Error};
use std::time::Instant;

pub(super) struct ConnectionQueue<C> {
    idle: ArrayQueue<Idle<C>>,
    waiters: SegQueue<Sender<Idle<C>>>,
}

impl<C> ConnectionQueue<C> {
    pub fn new(max_size: u32) -> Self {
        Self {
            idle: ArrayQueue::new(max_size as usize),
            waiters: SegQueue::new(),
        }
    }

    pub fn num_idle(&self) -> usize {
        self.idle.len()
    }

    pub fn push(&self, conn: Floating<Idle<C>>) {
        let mut idle = conn.into_leakable();

        // Try waiters in (FIFO) order until one is still waiting ..
        while let Ok(waiter) = self.waiters.pop() {
            idle = match waiter.send(idle) {
                // successfully released
                Ok(()) => return,
                // waiter was dropped, try another one
                Err(live) => live,
            };
        }

        // .. if there were no waiters still waiting, just push the connection
        // back to the idle queue
        self.idle
            .push(idle)
            .expect("connections exceeded max_size in idle queue");
    }

    pub fn try_pop<'s>(&self, size: &'s PoolSize) -> Option<Floating<'s, Idle<C>>> {
        self.idle.pop().ok().map(|conn| Floating::new(conn, size))
    }

    pub async fn pop<'s>(
        &self,
        size: &'s PoolSize,
        deadline: Instant,
    ) -> crate::Result<Floating<'s, Idle<C>>> {
        if let Some(conn) = self.try_pop(size) {
            return Ok(conn);
        }

        // Too many open connections
        // Wait until one is available
        let (tx, rx) = channel();

        self.waiters.push(tx);

        let timeout = super::deadline_as_timeout(deadline)?;

        // don't sleep forever
        match crate::runtime::timeout(timeout, rx).await {
            // A connection was returned to the pool
            Ok(Ok(idle)) => Ok(Floating::new(idle, size)),

            // waiter was dropped, most likely reason is that the pool was closing
            // but the runtime could also be shutting down
            Ok(Err(_)) => Err(Error::PoolClosed),

            // Timed out waiting for a connection
            // Error is not forwarded as its useless context
            Err(_) => Err(Error::PoolTimedOut(None)),
        }
    }

    pub async fn dump(&self, size: &PoolSize)
    where
        C: Connection,
    {
        // clear the waiters queue
        while let Ok(_) = self.waiters.pop() {}

        // evict connections from the queue
        while size.current() > 0 {
            while let Some(idle) = self.try_pop(&size) {
                let _ = idle.close().await;
            }

            // moves us to the back of the task queue, hopefully behind other tasks
            // currently using our connections
            crate::runtime::yield_now().await
        }
    }
}
