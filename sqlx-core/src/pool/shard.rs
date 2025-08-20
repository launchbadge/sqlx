use event_listener::Event;
use std::cell::OnceCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{atomic, Arc};
use std::{array, iter};

use parking_lot::Mutex;

type ShardId = usize;
type ConnectionIndex = usize;

pub struct Sharded<T> {
    shards: Box<[Arc<Shard<[Arc<Mutex<Option<T>>>]>>]>,
    global_unlock_event: Event<(ShardId, ConnectionIndex)>,
}

type ArcMutexGuard<T> = parking_lot::ArcMutexGuard<parking_lot::RawMutex, T>;

pub struct ConnectedGuard<T> {
    locked: ArcMutexGuard<Option<T>>,
}

pub struct UnconnectedGuard<T> {
    locked: ArcMutexGuard<Option<T>>,
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
    locked_set: AtomicUsize,
    unlock_event: Event<ConnectionIndex>,
    connected_set: AtomicUsize,
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

impl<T> Sharded<T> {
    pub fn new(connections: usize, shards: usize) -> Sharded<T> {
        let shards = Params::calc(connections, shards)
            .shard_sizes()
            .map(|shard_size| Shard::new(shard_size, || Arc::new(Mutex::new(None))))
            .collect::<Box<[_]>>();

        Sharded {
            shards,
            global_unlock_event: Event::with_tag(),
        }
    }

    pub async fn lock_connected(&self) -> ConnectedGuard<T> {}

    pub async fn lock_unconnected(&self) -> UnconnectedGuard<T> {}
}

impl<T> Shard<[T]> {
    fn new(len: usize, mut fill: impl FnMut() -> T) -> Arc<Shard<[T]>> {
        macro_rules! make_array {
            ($($n:literal),+) => {
                match len {
                    $($n => Arc::new(Shard {
                        locked_set: AtomicUsize::new(0),
                        unlock_event: Event::with_tag(),
                        connected_set: AtomicUsize::new(0),
                        connections: array::from_fn::<_, $n, _>(|_| fill())
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

    async fn acquire(&self, connected: bool) -> ArcMutexGuard<Option<T>> {
        if self.unlock_event.total_listeners() > 0 {}

        loop {
            let locked_set = self.locked_set.load(Ordering::Acquire);
            let connected_set = self.connected_set.load(Ordering::Relaxed);

            let connected_mask = if connected {
                connected_set
            } else {
                !connected_set
            };

            let index = (locked_set & connected_mask).trailing_zeros() as usize;

            if let Some(guard) = self.try_lock(index) {
                return guard;
            }
        }
    }

    fn try_lock(&self, index: ConnectionIndex) -> Option<ArcMutexGuard<Option<T>>> {}
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
    static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

    thread_local! {
        static CURRENT_THREAD_ID: usize = {
            THREAD_ID.fetch_add(1, Ordering::SeqCst)
        };
    }

    CURRENT_THREAD_ID.with(|i| *i)
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
