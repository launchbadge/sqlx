use event_listener::{Event, IntoNotification};
use parking_lot::Mutex;
use std::cell::OnceCell;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{atomic, Arc};
use std::{array, cmp, iter};

type ShardId = usize;
type ConnectionIndex = usize;

pub struct Shards<T> {
    shards: Box<[ArcShard<T>]>,
    global: Arc<Global>,
}

type ArcShard<T> = Arc<Shard<[Arc<Mutex<Option<T>>>]>>;

struct Global {
    unlock_event: Event<(ShardId, ConnectionIndex)>,
    disconnect_event: Event<(ShardId, ConnectionIndex)>,
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
struct Shard<T: ?Sized> {
    shard_id: ShardId,
    locked_set: AtomicUsize,
    unlock_event: Event<ConnectionIndex>,
    connected_set: AtomicUsize,
    disconnect_event: Event<ConnectionIndex>,
    global: Arc<Global>,
    connections: T,
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

impl<T> Shards<T> {
    pub fn new(connections: usize, shards: usize) -> Shards<T> {
        let global = Arc::new(Global {
            unlock_event: Event::with_tag(),
            disconnect_event: Event::with_tag(),
        });

        let shards = Params::calc(connections, shards)
            .shard_sizes()
            .enumerate()
            .map(|(shard_id, size)| Shard::new(shard_id, size, global.clone()))
            .collect::<Box<[_]>>();

        Shards { shards, global }
    }

    pub async fn acquire(&self, connected: bool) -> LockGuard<T> {}
}

impl<T> Shard<[Arc<Mutex<Option<T>>>]> {
    fn new(shard_id: ShardId, len: usize, global: Arc<Global>) -> Arc<Self> {
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
        loop {
            let locked_set = self.locked_set.load(Ordering::Acquire);
            let connected_set = self.connected_set.load(Ordering::Relaxed);

            let connected_mask = if connected {
                connected_set
            } else {
                !connected_set
            };

            // Choose the first index that is unlocked with bit `connected`
            let index = (!locked_set & connected_mask).leading_zeros() as usize;

            if let Some(guard) = self.try_lock(index) {
                return guard;
            }

            let index = if connected {
                event_listener::listener!(self.unlock_event => unlocked);
                unlocked.await
            } else {
                event_listener::listener!(self.disconnect_event => disconnected);
                disconnected.await
            };

            if let Some(guard) = self.try_lock(index) {
                return guard;
            }
        }
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

        // Release the lock.
        drop(locked);

        // Updating the connected flag shouldn't require a fence.
        atomic_set(
            &self.shard.connected_set,
            self.index,
            connected,
            Ordering::Relaxed,
        );

        if connected {
            // If another receiver is waiting for a connection on this shard,
            // we notify them without updating `locked_set`, effectively passing the lock to them.
            //
            // This prevents drive-by tasks from acquiring connections before waiting tasks
            // at high contention, while requiring little synchronization otherwise.
            //
            // However, we need to be careful to release the connection if the listener
            // that received it is dropped before it can complete the acquisition.
            if self.shard.unlock_event.notify(1.tag(self.index)) > 0 {
                return;
            }

            if self
                .shard
                .global
                .unlock_event
                .notify(1.tag((self.shard.shard_id, self.index)))
                > 0
            {
                return;
            }
        } else {
            if self.shard.disconnect_event.notify(1.tag(self.index)) > 0 {
                return;
            }

            if self
                .shard
                .global
                .disconnect_event
                .notify(1.tag((self.shard.shard_id, self.index)))
                > 0
            {
                return;
            }
        }

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
