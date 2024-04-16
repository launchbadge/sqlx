//! A cooperatively bounded SPSC channel.
//!
//! Senders may either obey the channel capacity but will have to wait when it is exhausted
//! ([`Sender::send`]) or ignore the channel capacity when necessary ([`Sender::send_unbounded()`]),
//! e.g. in a `Drop` impl where neither blocking nor waiting is acceptable.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;

use futures_util::task::AtomicWaker;

use crate::sync::{AsyncMutex, AsyncSemaphore};

/// The sender side of a cooperatively bounded SPSC channel.
///
/// The channel is closed when either this or [`Receiver`] is dropped.
pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

/// The receiver side of a cooperatively bounded SPSC channel.
///
/// The channel is closed when either this or [`Sender`] is dropped.
pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

struct Channel<T> {
    buffer: Buffer<T>,
    closed: AtomicBool,
    capacity: usize,
    semaphore: AsyncSemaphore,
    recv_waker: AtomicWaker,
    // Ensure we only attempt to read from the channel if this was a legitimate wakeup.
    receiver_woken: AtomicBool,
}

struct Buffer<T> {
    // Using double-buffering so the sender and receiver aren't constantly fighting
    front: AsyncMutex<Deque<T>>,
    back: AsyncMutex<Deque<T>>,
    write_front: AtomicBool,
}

struct Deque<T> {
    messages: VecDeque<T>,
    /// Each value represents the number of bounded sends at the head of the queue.
    ///
    /// Pushing a message that used a permit should increment the value at the back of this queue,
    /// or push a new value of 1 if the value is 255 or 0, or the queue is empty.
    ///
    /// Pushing a message that did _not_ use a permit (unbounded send) should simultaneously
    /// push a zero value to this queue.
    ///
    /// Popping a message should decrement the value at the head of this queue,
    /// removing it when it reaches 0. The receiver should simultaneously add a permit
    /// back to the channel semaphore.
    ///
    /// A 0 value at the head of the queue thus indicates an unbounded send,
    /// which should _not_ result in the release of a permit.
    permits_used: VecDeque<u8>,
}

impl<T> Sender<T> {
    /// Send a message or wait for a permit to be released from the receiver.
    ///
    /// ### Cancel-Safe
    /// This method is entirely cancel-safe. It is guaranteed to _only_ send the message
    /// immediately before it returns.
    ///
    /// Contrast this to [`flume::Sender::send_async()`][flume-cancel-safe], where
    /// the message _may_ be received any time after the first poll if it does not return `Ready`
    /// immediately.
    ///
    /// [flume-cancel-safe]: https://github.com/zesterer/flume/issues/104#issuecomment-1216387210
    pub async fn send(&mut self, message: T) -> Result<(), T> {
        if self.channel.closed.load(Ordering::Acquire) {
            return Err(message);
        }

        let permit = self.channel.semaphore.acquire(1).await;

        self.send_inner(message, true).map(|_| permit.consume())
    }

    /// Send a message immediately.
    ///
    /// ### Note: Only Use when Necessary
    /// This call ignores channel capacity and should only be used where blocking or waiting
    /// is not an option, e.g. in a `Drop` impl.
    pub fn send_unbounded(&mut self, message: T) -> Result<(), T> {
        self.send_inner(message, false)
    }

    fn send_inner(&mut self, message: T, permit_used: bool) -> Result<(), T> {
        if self.channel.closed.load(Ordering::Acquire) {
            return Err(message);
        }

        self.channel.buffer.write(message, permit_used);
        self.channel.receiver_woken.store(true, Ordering::Release);
        self.channel.recv_waker.wake();
        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.channel.closed.store(true, Ordering::Release);
        self.channel.recv_waker.wake();
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        loop {
            if self.channel.closed.load(Ordering::Acquire) {
                return None;
            }

            if let Some(message) = self.channel.buffer.read().await {
                return Some(message);
            }

            futures_util::future::poll_fn(|cx| {
                let ready = self.channel.closed.load(Ordering::Acquire)
                    || self.channel.receiver_woken.load(Ordering::Acquire);

                // Clear the `receiver_woken` flag.
                self.channel.receiver_woken.store(false, Ordering::Release);

                if ready {
                    Poll::Ready(())
                } else {
                    // Ensure the waker is up-to-date every time we're polled.
                    self.channel.recv_waker.register(cx.waker());
                    Poll::Pending
                }
            })
            .await;
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.channel.closed.store(true, Ordering::Relaxed);
        self.channel.recv_waker.take();
        self.channel.semaphore.release(self.channel.capacity);
    }
}

impl<T> Buffer<T> {
    pub fn write(&self, value: T, permit_used: bool) {
        let mut side = if self.write_front.load(Ordering::Acquire) {
            self.front
                .try_lock()
                .expect("BUG: receiver has front buffer locked while reading back buffer")
        } else {
            self.back
                .try_lock()
                .expect("BUG: receiver has back buffer locked while reading front buffer")
        };

        side.messages.push_back(value);

        if permit_used {
            side.permits_used = side.permits_used.checked_add(1)
                .expect("BUG: permits_used overflowed!");
        }
    }

    pub async fn read(&self) -> Option<(T, bool)> {
        // If the sender is writing the front, we should read the back and vice versa.
        let read_back = self.write_front.load(Ordering::Acquire);

        let mut side = if read_back {
            // If we just swapped buffers, we may need to wait for the sender to release the lock.
            self.back.lock().await
        } else {
            self.front.lock().await
        };

        let val = side.messages.pop_front();

        // It doesn't actually matter if this exact message actually used a permit,
        // it just matters that we made room in the channel.
        let permit_used = side.permits_used.checked_sub(1)
            .is_some_and(|permits_used| {
                side.permits_used = permits_used;
                true
            });

        // Note: be sure to release the lock before swapping or `write()` will panic.
        drop(side);

        if val.is_none() {
            // This side is empty; swap.
            self.write_front.store(!read_back, Ordering::Release);
        }

        val.map(|val| (val, permit_used))
    }
}

pub fn channel<T>(bounded_capacity: usize) -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel {
        buffer: Buffer {
            front: AsyncMutex::new(Deque {
                messages: VecDeque::with_capacity(bounded_capacity),
                permits_used: 0,
            }),
            back: AsyncMutex::new(Deque {
                messages: VecDeque::with_capacity(bounded_capacity),
                permits_used: 0,
            }),
            write_front: true.into()
        },
        closed: false.into(),
        capacity: bounded_capacity,
        semaphore: AsyncSemaphore::new(true, bounded_capacity),
        recv_waker: Default::default(),
        receiver_woken: false.into(),
    });

    (
        Sender {
            channel: channel.clone(),
        },
        Receiver {
            channel
        }
    )
}
