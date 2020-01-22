use std::mem;
use std::sync::atomic::{AtomicU32, Ordering};

pub(in crate::pool) struct PoolSize {
    size: AtomicU32,
    max_size: u32,
}

pub(in crate::pool) struct IncreaseGuard<'a>(DecreaseOnDrop<'a>);

pub(in crate::pool) struct DecreaseOnDrop<'a> {
    size: &'a AtomicU32,
    dropped: bool,
}

impl PoolSize {
    pub fn new(max_size: u32) -> Self {
        PoolSize {
            size: AtomicU32::new(0),
            max_size,
        }
    }

    pub fn current(&self) -> u32 {
        self.size.load(Ordering::Acquire)
    }

    pub fn try_increase(&self) -> Option<IncreaseGuard> {
        let mut size = self.current();

        while size < self.max_size {
            // we want to stop at size == max_size
            let new_size = self.size.compare_and_swap(size, size + 1, Ordering::AcqRel);

            if new_size == size + 1 {
                return Some(IncreaseGuard(self.decrease_on_drop()));
            }

            size = new_size;
        }

        None
    }

    pub fn decrease_on_drop(&self) -> DecreaseOnDrop {
        DecreaseOnDrop {
            size: &self.size,
            dropped: false,
        }
    }
}

impl IncreaseGuard<'_> {
    pub fn commit(self) {
        self.0.cancel();
    }
}

impl DecreaseOnDrop<'_> {
    pub fn cancel(self) {
        mem::forget(self);
    }
}

impl Drop for DecreaseOnDrop<'_> {
    fn drop(&mut self) {
        assert!(!self.dropped, "double-dropped!");
        self.dropped = true;
        self.size.fetch_sub(1, Ordering::AcqRel);
    }
}
