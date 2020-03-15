use crossbeam_queue::ArrayQueue;
use futures_channel::oneshot;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{park, spawn, JoinHandle};

// After tinkering with this, I believe the safest solution is to spin up a discrete thread per
// SQLite connection and perform all I/O operations for SQLite on _that_ thread. To this effect
// we have a worker struct that is a thin message passing API to run messages on the worker thread.

#[derive(Clone)]
pub(crate) struct Worker {
    running: Arc<AtomicBool>,
    queue: Arc<ArrayQueue<Box<dyn FnOnce() + Send>>>,
    handle: Arc<JoinHandle<()>>,
}

impl Worker {
    pub(crate) fn new() -> Self {
        let queue: Arc<ArrayQueue<Box<dyn FnOnce() + Send>>> = Arc::new(ArrayQueue::new(1));
        let running = Arc::new(AtomicBool::new(true));

        Self {
            handle: Arc::new(spawn({
                let queue = queue.clone();
                let running = running.clone();

                move || {
                    while running.load(Ordering::SeqCst) {
                        if let Ok(message) = queue.pop() {
                            (message)();
                        }

                        park();
                    }
                }
            })),
            queue,
            running,
        }
    }

    pub(crate) async fn run<F, R>(&mut self, f: F) -> R
    where
        F: Send + 'static,
        R: Send + 'static,
        F: FnOnce() -> R,
    {
        let (sender, receiver) = oneshot::channel::<R>();

        let _ = self.queue.push(Box::new(move || {
            let _ = sender.send(f());
        }));

        self.handle.thread().unpark();

        receiver.await.unwrap()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if Arc::strong_count(&self.handle) == 1 {
            self.running.store(false, Ordering::SeqCst);
        }
    }
}
