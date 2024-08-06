use std::collections::VecDeque;
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;

use futures_util::task::AtomicWaker;

pub struct Sender<T> {
    shared: Arc<BufferShared<T>>,
    buffer: BufferOption<T>,
}

pub struct Receiver<T> {
    shared: Arc<BufferShared<T>>,
    buffer: BufferOption<T>,
}

struct BufferShared<T> {
    header: Header,
    // Instead of writing to buffers in shared memory, which would require up to
    // 128 bytes of padding to prevent false sharing, the sender and receiver each take
    // exclusive ownership of the buffer they're currently accessing.
    //
    // This way, contended access to shared memory only happens when it's time for a buffer swap.
    front: Mutex<Option<VecDeque<T>>>,
    back: Mutex<Option<VecDeque<T>>>,
}

enum BufferOption<T> {
    Wants(SelectedBuffer),
    HasFront(VecDeque<T>),
    HasBack(VecDeque<T>),
}

#[derive(Debug)]
struct Header {
    sender_waiting: AtomicWaker,
    receiver_waiting: AtomicWaker,

    closed: AtomicBool,

    front_flushed: AtomicBool,
    back_flushed: AtomicBool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum SelectedBuffer {
    Front,
    Back,
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let buffer_capacity = capacity / 2;
    assert_ne!(buffer_capacity, 0, "capacity / 2 must not be zero");

    // Sender starts out owning the front buffer,
    // receiver starts out _wanting_ the front buffer.
    let shared = Arc::new(BufferShared {
        header: Header {
            closed: AtomicBool::new(false),
            front_flushed: AtomicBool::new(false),
            back_flushed: AtomicBool::new(false),
            sender_waiting: AtomicWaker::new(),
            receiver_waiting: AtomicWaker::new(),
        },
        front: Mutex::new(None),
        back: Mutex::new(Some(VecDeque::with_capacity(buffer_capacity))),
    });

    (
        Sender {
            shared: shared.clone(),
            buffer: BufferOption::HasFront(VecDeque::with_capacity(buffer_capacity)),
        },
        Receiver {
            shared: shared.clone(),
            buffer: BufferOption::Wants(SelectedBuffer::Front),
        }
    )
}

impl<T> Sender<T> {
    /// Flush the current buffer and wake the reader.
    fn flush_buffer(&mut self) {
        let selected = self.buffer.as_selected();

        let Some(buf) = mem::replace(&mut self.buffer, BufferOption::Wants(selected.next()))
            .into_buf() else {
            return;
        };

        self.shared.put_buffer(selected, buf);

        self.shared.header.flushed_status(selected)
            .store(true, Ordering::Release);

        self.shared.header.receiver_waiting.wake();
    }

    pub async fn send(&mut self, val: T) -> Result<(), T> {
        loop {
            if self.shared.header.is_closed() {
                return Err(val);
            }

            let selected = self.buffer.as_selected();
            let flushed_status = self.shared.header.flushed_status(selected);

            if let Some(buf) = self.buffer.get_mut() {
                buf.push_back(val);

                if buf.len() == buf.capacity() {
                    // Advances to the next buffer.
                    self.flush_buffer();
                }

                return Ok(());
            }

            let res = std::future::poll_fn(|cx| {
                self.shared.header.sender_waiting.register(cx.waker());

                if self.shared.header.is_closed() {
                    return Poll::Ready(Err(()));
                }

                if flushed_status.load(Ordering::Acquire) {
                    return Poll::Pending
                }

                Poll::Ready(Ok(()))
            }).await;

            if let Err(()) = res {
                return Err(val);
            }

            let buf = self.shared.take_buffer(self.buffer.as_selected());
            self.buffer.put(buf);
        }
    }
}

/// Closes the channel.
///
/// The receiver may continue to read messages until the channel is drained.
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.flush_buffer();
        self.shared.header.close();
    }
}

impl<T> Receiver<T> {
    fn release_buffer(&mut self) {
        let selected = self.buffer.as_selected();

        let Some(buf) = mem::replace(&mut self.buffer, BufferOption::Wants(selected.next()))
            .into_buf() else {
            return;
        };

        self.shared.put_buffer(selected, buf);

        self.shared.header.flushed_status(selected)
            .store(false, Ordering::Release);

        self.shared.header.sender_waiting.wake();
    }


    pub async fn recv(&mut self) -> Option<T> {
        loop {
            // Note: we don't check if the channel is closed until we swap buffers.
            if let Some(buf) = self.buffer.get_mut() {
                if let Some(val) = buf.pop_front() {
                    if buf.is_empty() {
                        self.release_buffer();
                    }

                    return Some(val);
                }

                // This *should* be a no-op, but it doesn't hurt to check again.
                self.release_buffer();
            }

            let flushed_status = self.shared.header.flushed_status(self.buffer.as_selected());

            std::future::poll_fn(|cx| {
                self.shared.header.receiver_waiting.register(cx.waker());

                // Sender has flushed this buffer.
                if flushed_status.load(Ordering::Acquire) {
                    return Poll::Ready(Some(()));
                }

                // Allow the reader to drain messages until the channel is empty.
                if self.shared.header.is_closed() {
                    return Poll::Ready(None);
                }

                // Waiting for the sender to write to and flush this buffer.
                Poll::Pending
            }).await?;

            let buf = self.shared.take_buffer(self.buffer.as_selected());
            self.buffer.put(buf);
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Unlike
        self.shared.header.close();
    }
}

impl Header {
    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.sender_waiting.wake();
        self.receiver_waiting.wake();
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }


    fn flushed_status(&self, buffer: SelectedBuffer) -> &AtomicBool {
        match buffer {
            SelectedBuffer::Front => &self.front_flushed,
            SelectedBuffer::Back => &self.back_flushed,
        }
    }
}

impl<T> BufferShared<T> {
    fn lock_buffer_place(&self, buffer: SelectedBuffer) -> MutexGuard<'_, Option<VecDeque<T>>> {
        match buffer {
            SelectedBuffer::Front => &self.front,
            SelectedBuffer::Back => &self.back,
        }
            .lock()
            .unwrap_or_else(|it| it.into_inner())
    }

    fn take_buffer(&self, selected: SelectedBuffer) -> VecDeque<T> {
        self
            .lock_buffer_place(selected)
            .take()
            .unwrap_or_else(|| panic!("expected to take {selected:?}, found nothing"))
    }

    fn put_buffer(&self, selected: SelectedBuffer, buf: VecDeque<T>) {
        let replaced = mem::replace(&mut *self.lock_buffer_place(selected), Some(buf));

        if let Some(replaced) = replaced {
            panic!("BUG: replaced buffer {selected:?} with {} elements", replaced.len());
        }
    }
}

impl<T> BufferOption<T> {
    fn as_selected(&self) -> SelectedBuffer {
        match *self {
            Self::Wants(wants) => wants,
            Self::HasFront(_) => SelectedBuffer::Front,
            Self::HasBack(_) => SelectedBuffer::Back,
        }
    }

    fn get_mut(&mut self) -> Option<&mut VecDeque<T>> {
        match self {
            Self::HasFront(front) => Some(front),
            Self::HasBack(back) => Some(back),
            _ => None,
        }
    }

    fn put(&mut self, buf: VecDeque<T>) {
        match self {
            Self::Wants(SelectedBuffer::Front) => *self = Self::HasFront(buf),
            Self::Wants(SelectedBuffer::Back) => *self = Self::HasBack(buf),
            Self::HasFront(front) => {
                panic!("BUG: replacing front buffer of len {} with buffer of len {}", front.len(), buf.len());
            }
            Self::HasBack(back) => {
                panic!("BUG: replacing back buffer of len {} with buffer of len {}", back.len(), buf.len());
            }
        }
    }

    fn into_buf(self) -> Option<VecDeque<T>> {
        match self {
            Self::HasFront(front) => Some(front),
            Self::HasBack(back) => Some(back),
            _ => None,
        }
    }
}

impl SelectedBuffer {
    fn next(&self) -> Self {
        match self {
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }
}


#[cfg(all(test, any(feature = "_rt-tokio", feature = "_rt-async-std")))]
mod tests {
    // Cannot use `#[sqlx::test]` because we want to configure the Tokio runtime to use 2 threads
    #[cfg(feature = "_rt-tokio")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_double_buffer_tokio() {
        test_double_buffer().await;
    }

    #[cfg(feature = "_rt-async-std")]
    #[async_std::test]
    async fn test_double_buffer_async_std() {
        test_double_buffer().await;
    }

    async fn test_double_buffer() {
        const CAPACITY: usize = 50;
        const END: usize = 1000;

        let (mut tx, mut rx) = super::channel::<usize>(CAPACITY);

        let reader = crate::rt::spawn(async move {
            for expected in 0usize..=END {
                assert_eq!(rx.recv().await, Some(expected));
            }

            assert_eq!(rx.recv().await, None)
        });

        let writer = crate::rt::spawn(async move {
            for val in 0usize..=END {
                tx.send(val).await.expect("buffer closed prematurely")
            }
        });

        // Our wrapper for `JoinHandle` propagates panics in both cases
        futures_util::future::join(
            reader,
            writer,
        ).await;
    }
}
