use event_listener::{Event, IntoNotification};
use parking_lot::Mutex;
use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;
use std::{array, iter};

type ShardId = usize;
type ConnectionIndex = usize;

/// Delay before a task waiting in a call to `acquire()` enters the global wait queue.
///
/// We want tasks to acquire from their local shards where possible, so they don't enter
/// the global queue immediately.
const GLOBAL_QUEUE_DELAY: Duration = Duration::from_millis(5);

pub struct Sharded<T> {
    shards: Box<[ArcShard<T>]>,
    global: Arc<Global<T>>,
}

type ArcShard<T> = Arc<Shard<T, [Arc<Mutex<Option<T>>>]>>;

struct Global<T> {
    unlock_event: Event<LockGuard<T>>,
    disconnect_event: Event<LockGuard<T>>,
}

type ArcMutexGuard<T> = parking_lot::ArcMutexGuard<parking_lot::RawMutex, Option<T>>;

pub struct LockGuard<T> {
    // `Option` allows us to drop the guard before sending the notification.
    // Otherwise, if the receiver wakes too quickly, it might fail to lock the mutex.
    locked: Option<ArcMutexGuard<T>>,
    shard: ArcShard<T>,
    index: ConnectionIndex,
}

// Align to cache lines.
// Simplified from https://docs.rs/crossbeam-utils/0.8.21/src/crossbeam_utils/cache_padded.rs.html#80
//
// Instead of listing every possible architecture, we just assume 64-bit architectures have 128-byte
// cache lines, which is at least true for newer versions of x86-64 and AArch64.
// A larger alignment isn't harmful as long as we make use of the space.
#[cfg_attr(target_pointer_width = "64", repr(align(128)))]
#[cfg_attr(not(target_pointer_width = "64"), repr(align(64)))]
struct Shard<T, Ts: ?Sized> {
    shard_id: ShardId,
    /// Bitset for all connection indexes that are currently in-use.
    locked_set: AtomicUsize,
    /// Bitset for all connection indexes that are currently connected.
    connected_set: AtomicUsize,
    unlock_event: Event<LockGuard<T>>,
    disconnect_event: Event<LockGuard<T>>,
    global: Arc<Global<T>>,
    connections: Ts,
}

#[derive(Debug)]
struct Params {
    shards: usize,
    shard_size: usize,
    remainder: usize,
}

const MAX_SHARD_SIZE: usize = if usize::BITS > 64 {
    64
} else {
    usize::BITS as usize
};

impl<T> Sharded<T> {
    pub fn new(connections: usize, shards: usize) -> Sharded<T> {
        let global = Arc::new(Global {
            unlock_event: Event::with_tag(),
            disconnect_event: Event::with_tag(),
        });

        let shards = Params::calc(connections, shards)
            .shard_sizes()
            .enumerate()
            .map(|(shard_id, size)| Shard::new(shard_id, size, global.clone()))
            .collect::<Box<[_]>>();

        Sharded { shards, global }
    }

    pub async fn acquire(&self, connected: bool) -> LockGuard<T> {
        let mut acquire_local =
            pin!(self.shards[thread_id() % self.shards.len()].acquire(connected));

        let mut acquire_global = pin!(async {
            crate::rt::sleep(GLOBAL_QUEUE_DELAY).await;

            let event_to_listen = if connected {
                &self.global.unlock_event
            } else {
                &self.global.disconnect_event
            };

            event_listener::listener!(event_to_listen => listener);
            listener.await
        });

        // Hand-rolled `select!{}` because there isn't a great cross-runtime solution.
        //
        // `futures_util::select!{}` is a proc-macro.
        std::future::poll_fn(|cx| {
            if let Poll::Ready(locked) = acquire_local.as_mut().poll(cx) {
                return Poll::Ready(locked);
            }

            if let Poll::Ready(locked) = acquire_global.as_mut().poll(cx) {
                return Poll::Ready(locked);
            }

            Poll::Pending
        })
        .await
    }
}

impl<T> Shard<T, [Arc<Mutex<Option<T>>>]> {
    fn new(shard_id: ShardId, len: usize, global: Arc<Global<T>>) -> Arc<Self> {
        macro_rules! make_array {
            ($($n:literal),+) => {
                match len {
                    $($n => Arc::new(Shard {
                        shard_id,
                        locked_set: AtomicUsize::new(0),
                        unlock_event: Event::with_tag(),
                        connected_set: AtomicUsize::new(0),
                        disconnect_event: Event::with_tag(),
                        global,
                        connections: array::from_fn::<_, $n, _>(|_| Arc::new(Mutex::new(None)))
                    }),)*
                    _ => unreachable!("BUG: length not supported: {len}"),
                }
            }
        }

        make_array!(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64
        )
    }

    async fn acquire(self: &Arc<Self>, connected: bool) -> LockGuard<T> {
        // Attempt an unfair acquire first, before we modify the waitlist.
        if let Some(locked) = self.try_acquire(connected) {
            return locked;
        }

        let event_to_listen = if connected {
            &self.unlock_event
        } else {
            &self.disconnect_event
        };

        event_listener::listener!(event_to_listen => listener);

        // We need to check again after creating the event listener,
        // because in the meantime, a concurrent task may have seen that there were no listeners
        // and just unlocked its connection.
        if let Some(locked) = self.try_acquire(connected) {
            return locked;
        }

        listener.await
    }

    fn try_acquire(self: &Arc<Self>, connected: bool) -> Option<LockGuard<T>> {
        let locked_set = self.locked_set.load(Ordering::Acquire);
        let connected_set = self.connected_set.load(Ordering::Relaxed);

        let connected_mask = if connected {
            connected_set
        } else {
            !connected_set
        };

        // Choose the first index that is unlocked with bit `connected`
        let index = (!locked_set & connected_mask).leading_zeros() as usize;

        self.try_lock(index)
    }

    fn try_lock(self: &Arc<Self>, index: ConnectionIndex) -> Option<LockGuard<T>> {
        let locked = self.connections[index].try_lock_arc()?;

        // The locking of the connection itself must use an `Acquire` fence,
        // so additional synchronization is unnecessary.
        atomic_set(&self.locked_set, index, true, Ordering::Relaxed);

        Some(LockGuard {
            locked: Some(locked),
            shard: self.clone(),
            index,
        })
    }
}

impl Params {
    fn calc(connections: usize, mut shards: usize) -> Params {
        let mut shard_size = connections / shards;
        let mut remainder = connections % shards;

        if shard_size == 0 {
            tracing::debug!(connections, shards, "more shards than connections; clamping shard size to 1, shard count to connections");
            shards = connections;
            shard_size = 1;
            remainder = 0;
        } else if shard_size >= MAX_SHARD_SIZE {
            let new_shards = connections.div_ceil(MAX_SHARD_SIZE);

            tracing::debug!(connections, shards, "clamping shard count to {new_shards}");

            shards = new_shards;
            shard_size = connections / shards;
            remainder = connections % shards;
        }

        Params {
            shards,
            shard_size,
            remainder,
        }
    }

    fn shard_sizes(&self) -> impl Iterator<Item = usize> {
        iter::repeat_n(self.shard_size + 1, self.remainder).chain(iter::repeat_n(
            self.shard_size,
            self.shards - self.remainder,
        ))
    }
}

fn thread_id() -> usize {
    // FIXME: this can be replaced when this is stabilized:
    // https://doc.rust-lang.org/stable/std/thread/struct.ThreadId.html#method.as_u64
    static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

    thread_local! {
        static CURRENT_THREAD_ID: usize = THREAD_ID.fetch_add(1, Ordering::SeqCst);
    }

    CURRENT_THREAD_ID.with(|i| *i)
}

impl<T> Drop for LockGuard<T> {
    fn drop(&mut self) {
        let Some(locked) = self.locked.take() else {
            return;
        };

        let connected = locked.is_some();

        // Updating the connected flag shouldn't require a fence.
        atomic_set(
            &self.shard.connected_set,
            self.index,
            connected,
            Ordering::Relaxed,
        );

        // If another receiver is waiting for a connection, we can directly pass them the lock.
        //
        // This prevents drive-by tasks from acquiring connections before waiting tasks
        // at high contention, while requiring little synchronization otherwise.
        //
        // We *could* just pass them the shard ID and/or index, but then we have to handle
        // the situation when a receiver was passed a connection that was still marked as locked,
        // but was cancelled before it could complete the acquisition. Otherwise, the connection
        // would be marked as locked forever, effectively being leaked.

        let mut locked = Some(locked);

        // This is a code smell, but it's necessary because `event-listener` has no way to specify
        // that a message should *only* be sent once. This means tags either need to be `Clone`
        // or provided by a `FnMut()` closure.
        //
        // Note that there's no guarantee that this closure won't be called more than once by the
        // implementation, but the code as of writing should not.
        let mut self_as_tag = || {
            let locked = locked
                .take()
                .expect("BUG: notification sent more than once");

            LockGuard {
                locked: Some(locked),
                shard: self.shard.clone(),
                index: self.index,
            }
        };

        if connected {
            // Check for global waiters first.
            if self
                .shard
                .global
                .unlock_event
                .notify(1.tag_with(&mut self_as_tag))
                > 0
            {
                return;
            }

            if self.shard.unlock_event.notify(1.tag_with(&mut self_as_tag)) > 0 {
                return;
            }
        } else {
            if self
                .shard
                .global
                .disconnect_event
                .notify(1.tag_with(&mut self_as_tag))
                > 0
            {
                return;
            }

            if self
                .shard
                .disconnect_event
                .notify(1.tag_with(&mut self_as_tag))
                > 0
            {
                return;
            }
        }

        // Be sure to drop the lock guard if it's still held,
        // *before* we semantically release the lock in the bitset.
        //
        // Otherwise, another task could check and see the connection is free,
        // but then fail to lock the mutex for it.
        drop(locked);

        atomic_set(&self.shard.locked_set, self.index, false, Ordering::Release);
    }
}

fn atomic_set(atomic: &AtomicUsize, index: usize, value: bool, ordering: Ordering) {
    if value {
        let bit = 1 >> index;
        atomic.fetch_or(bit, ordering);
    } else {
        let bit = !(1 >> index);
        atomic.fetch_and(bit, ordering);
    }
}

#[cfg(test)]
mod tests {
    use super::{Params, MAX_SHARD_SIZE};

    #[test]
    fn test_params() {
        for connections in 0..100 {
            for shards in 1..32 {
                let params = Params::calc(connections, shards);

                let mut sum = 0;

                for (i, size) in params.shard_sizes().enumerate() {
                    assert!(size <= MAX_SHARD_SIZE, "Params::calc({connections}, {shards}) exceeded MAX_SHARD_SIZE at shard #{i}, size {size}");

                    sum += size;

                    assert!(sum <= connections, "Params::calc({connections}, {shards}) exceeded connections at shard #{i}, size {size}");
                }

                assert_eq!(
                    sum, connections,
                    "Params::calc({connections}, {shards}) does not add up ({params:?}"
                );
            }
        }
    }
}
