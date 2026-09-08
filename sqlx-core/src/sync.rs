use asyncband::semaphore::{Semaphore, SemaphorePermit};

pub struct AsyncSemaphore {
    inner: Semaphore,
}

impl AsyncSemaphore {
    #[track_caller]
    pub fn new(permits: usize) -> Self {
        if cfg!(not(any(
            feature = "_rt-async-global-executor",
            feature = "_rt-async-std",
            feature = "_rt-smol",
            feature = "_rt-tokio"
        ))) {
            crate::rt::missing_rt(permits);
        }

        Self {
            inner: Semaphore::new(permits),
        }
    }

    pub fn permits(&self) -> usize {
        self.inner.available_permits()
    }

    pub async fn acquire(&self, permits: u32) -> AsyncSemaphoreReleaser<'_> {
        AsyncSemaphoreReleaser {
            inner: self.inner.acquire(permits as usize).await,
        }
    }

    pub fn try_acquire(&self, permits: u32) -> Option<AsyncSemaphoreReleaser<'_>> {
        Some(AsyncSemaphoreReleaser {
            inner: self.inner.try_acquire(permits as usize)?,
        })
    }

    pub fn release(&self, permits: usize) {
        self.inner.release(permits);
    }
}

pub struct AsyncSemaphoreReleaser<'a> {
    inner: SemaphorePermit<'a>,
}

impl AsyncSemaphoreReleaser<'_> {
    pub fn disarm(self) {
        self.inner.forget();
    }
}
