use crate::ext::future::race;
use crate::rt;
use crate::sync::{AsyncMutex, AsyncMutexGuardArc};
use event_listener::{listener, Event, EventListener, IntoNotification};
use futures_core::Stream;
use futures_util::stream::FuturesUnordered;
use futures_util::{FutureExt, StreamExt};
use std::cmp;
use std::future::Future;
use std::ops::{Deref, DerefMut, RangeInclusive, RangeToInclusive};
use std::pin::{pin, Pin};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

pub struct ConnectionSet<C> {
    global: Arc<Global>,
    slots: Box<[Arc<Slot<C>>]>,
}

pub struct ConnectedSlot<C>(SlotGuard<C>);

pub struct DisconnectedSlot<C>(SlotGuard<C>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AcquirePreference {
    Connected,
    Disconnected,
    Either,
}

struct Global {
    unlock_event: Event<usize>,
    disconnect_event: Event<usize>,
    locked_set: Box<[AtomicBool]>,
    num_connected: AtomicUsize,
    min_connections: usize,
    min_connections_event: Event<()>,
}

struct SlotGuard<C> {
    slot: Arc<Slot<C>>,
    // `Option` allows us to take the guard in the drop handler.
    locked: Option<AsyncMutexGuardArc<Option<C>>>,
}

struct Slot<C> {
    // By having each `Slot` hold its own reference to `Global`, we can avoid extra contended clones
    // which would sap performance
    global: Arc<Global>,
    index: usize,
    // I'd love to eliminate this redundant `Arc` but it's likely not possible without `unsafe`
    connection: Arc<AsyncMutex<Option<C>>>,
    unlock_event: Event,
    disconnect_event: Event,
    connected: AtomicBool,
    locked: AtomicBool,
    leaked: AtomicBool,
}

impl<C> ConnectionSet<C> {
    pub fn new(size: RangeInclusive<usize>) -> Self {
        let global = Arc::new(Global {
            unlock_event: Event::with_tag(),
            disconnect_event: Event::with_tag(),
            locked_set: (0..*size.end()).map(|_| AtomicBool::new(false)).collect(),
            num_connected: AtomicUsize::new(0),
            min_connections: *size.start(),
            min_connections_event: Event::with_tag(),
        });

        ConnectionSet {
            // `vec![<expr>; size].into()` clones `<expr>` instead of repeating it,
            // which is *no bueno* when wrapping something in `Arc`
            slots: (0..*size.end())
                .map(|index| {
                    Arc::new(Slot {
                        global: global.clone(),
                        index,
                        connection: Arc::new(AsyncMutex::new(None)),
                        unlock_event: Event::with_tag(),
                        disconnect_event: Event::with_tag(),
                        connected: AtomicBool::new(false),
                        locked: AtomicBool::new(false),
                        leaked: AtomicBool::new(false),
                    })
                })
                .collect(),
            global,
        }
    }

    #[inline(always)]
    pub fn num_connected(&self) -> usize {
        self.global.num_connected()
    }

    pub fn count_idle(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_locked()).count()
    }

    pub async fn acquire_connected(&self) -> ConnectedSlot<C> {
        self.acquire_inner(AcquirePreference::Connected)
            .await
            .assert_connected()
    }

    pub async fn acquire_disconnected(&self) -> DisconnectedSlot<C> {
        self.acquire_inner(AcquirePreference::Disconnected)
            .await
            .assert_disconnected()
    }

    /// Attempt to acquire the connection associated with the current thread.
    pub async fn acquire_any(&self) -> Result<ConnectedSlot<C>, DisconnectedSlot<C>> {
        self.acquire_inner(AcquirePreference::Either)
            .await
            .try_connected()
    }

    async fn acquire_inner(&self, pref: AcquirePreference) -> SlotGuard<C> {
                let preferred_slot = current_thread_id() % self.slots.len();

        tracing::trace!(preferred_slot, ?pref, "acquire_inner");

        // Always try to lock the connection associated with our thread ID
        let mut acquire_preferred = pin!(self.slots[preferred_slot].acquire(pref));

        let mut listen_global = pin!(self.global.listen(pref));

        let mut yielded = false;

        std::future::poll_fn(|cx| {
            if let Poll::Ready(locked) = acquire_preferred.as_mut().poll(cx) {
                return Poll::Ready(locked);
            }

            if let Poll::Ready(slot) = listen_global.as_mut().poll(cx) {
                if let Some(locked) = self.slots[slot].try_acquire(pref) {
                    return Poll::Ready(locked);
                }

                listen_global.as_mut().set(self.global.listen(pref));
            }

            if !yielded {
                cx.waker().wake_by_ref();
                yielded = true;
                return Poll::Pending;
            }

            if let Some(locked) = self.try_acquire(pref) {
                return Poll::Ready(locked);
            }

            Poll::Pending
        })
        .await
    }

    pub fn try_acquire_connected(&self) -> Option<ConnectedSlot<C>> {
        Some(
            self.try_acquire(AcquirePreference::Connected)?
                .assert_connected(),
        )
    }

    pub fn try_acquire_disconnected(&self) -> Option<DisconnectedSlot<C>> {
        Some(
            self.try_acquire(AcquirePreference::Disconnected)?
                .assert_disconnected(),
        )
    }

    fn try_acquire(&self, pref: AcquirePreference) -> Option<SlotGuard<C>> {
        let mut search_slot = current_thread_id() % self.slots.len();

        for _ in 0..self.slots.len() {
            if let Some(locked) = self.slots[search_slot].try_acquire(pref) {
                return Some(locked);
            }

            search_slot = self.next_slot(search_slot);
        }

        None
    }

    pub fn min_connections_listener(&self) -> EventListener {
        self.global.min_connections_event.listen()
    }

    pub fn iter_idle(&self) -> impl Iterator<Item = ConnectedSlot<C>> + '_ {
        self.slots.iter().filter_map(|slot| {
            Some(
                slot.try_acquire(AcquirePreference::Connected)?
                    .assert_connected(),
            )
        })
    }

    pub async fn drain(&self, ref close: impl AsyncFn(ConnectedSlot<C>) -> DisconnectedSlot<C>) {
        let mut closing = FuturesUnordered::new();

        // We could try to be more efficient by only populating the `FuturesUnordered` for
        // connected slots, but then we'd have to handle a disconnected slot becoming connected,
        // which could happen concurrently.
        //
        // However, we don't *need* to be efficient when shutting down the pool.
        for slot in &self.slots {
            closing.push(async {
                let locked = slot.lock().await;

                let slot = match locked.try_connected() {
                    Ok(connected) => close(connected).await,
                    Err(disconnected) => disconnected,
                };

                // The pool is shutting down; don't wake any tasks that might have been interested
                slot.leak();
            });
        }

        while closing.next().await.is_some() {}
    }

    #[inline(always)]
    fn next_slot(&self, slot: usize) -> usize {
        // By adding a number that is coprime to `slots.len()` before taking the modulo,
        // we can visit each slot in a pseudo-random order, spreading the demand evenly.
        //
        // Interestingly, this pattern returns to the original slot after `slots.len()` iterations,
        // because of congruence: https://en.wikipedia.org/wiki/Modular_arithmetic#Congruence
        (slot + 547) % self.slots.len()
    }
}

impl AcquirePreference {
    #[inline(always)]
    fn wants_connected(&self, is_connected: bool) -> bool {
        match (self, is_connected) {
            (Self::Connected, true) => true,
            (Self::Disconnected, false) => true,
            (Self::Either, _) => true,
            _ => false,
        }
    }
}

impl<C> Slot<C> {
    #[inline(always)]
    fn matches_pref(&self, pref: AcquirePreference) -> bool {
        !self.is_leaked() && pref.wants_connected(self.is_connected())
    }

    #[inline(always)]
    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    #[inline(always)]
    fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    #[inline(always)]
    fn is_leaked(&self) -> bool {
        self.leaked.load(Ordering::Relaxed)
    }

    #[inline(always)]
    fn set_is_connected(&self, connected: bool) {
        let was_connected = self.connected.swap(connected, Ordering::Acquire);

        match (connected, was_connected) {
            (false, true) => {
                // Ensure this is synchronized with `connected`
                self.global.num_connected.fetch_add(1, Ordering::Release);
            }
            (true, false) => {
                self.global.num_connected.fetch_sub(1, Ordering::Release);
            }
            _ => (),
        }
    }

    async fn acquire(self: &Arc<Self>, pref: AcquirePreference) -> SlotGuard<C> {
        loop {
            if self.matches_pref(pref) {
                tracing::trace!(slot_index=%self.index, "waiting for lock");

                let locked = self.lock().await;

                if locked.matches_pref(pref) {
                    return locked;
                }
            }

            match pref {
                AcquirePreference::Connected => {
                    listener!(self.unlock_event => listener);
                    listener.await;
                }
                AcquirePreference::Disconnected => {
                    listener!(self.disconnect_event => listener);
                    listener.await
                }
                AcquirePreference::Either => {
                    listener!(self.unlock_event => unlock_listener);
                    listener!(self.disconnect_event => disconnect_listener);
                    race(unlock_listener, disconnect_listener).await.ok();
                }
            }
        }
    }

    fn try_acquire(self: &Arc<Self>, pref: AcquirePreference) -> Option<SlotGuard<C>> {
        if self.matches_pref(pref) {
            let locked = self.try_lock()?;

            if locked.matches_pref(pref) {
                return Some(locked);
            }
        }

        None
    }

    async fn lock(self: &Arc<Self>) -> SlotGuard<C> {
        let locked = crate::sync::lock_arc(&self.connection).await;

        self.locked.store(true, Ordering::Relaxed);

        SlotGuard {
            slot: self.clone(),
            locked: Some(locked),
        }
    }

    fn try_lock(self: &Arc<Self>) -> Option<SlotGuard<C>> {
        let locked = crate::sync::try_lock_arc(&self.connection)?;

        self.locked.store(true, Ordering::Relaxed);

        Some(SlotGuard {
            slot: self.clone(),
            locked: Some(locked),
        })
    }
}

impl<C> SlotGuard<C> {
    #[inline(always)]
    fn get(&self) -> &Option<C> {
        self.locked.as_ref().expect(EXPECT_LOCKED)
    }

    #[inline(always)]
    fn get_mut(&mut self) -> &mut Option<C> {
        self.locked.as_mut().expect(EXPECT_LOCKED)
    }

    #[inline(always)]
    fn matches_pref(&self, pref: AcquirePreference) -> bool {
        !self.slot.is_leaked() && pref.wants_connected(self.is_connected())
    }

    #[inline(always)]
    fn is_connected(&self) -> bool {
        self.get().is_some()
    }

    fn try_connected(self) -> Result<ConnectedSlot<C>, DisconnectedSlot<C>> {
        if self.is_connected() {
            Ok(ConnectedSlot(self))
        } else {
            Err(DisconnectedSlot(self))
        }
    }

    fn assert_connected(self) -> ConnectedSlot<C> {
        assert!(self.is_connected());
        ConnectedSlot(self)
    }

    fn assert_disconnected(self) -> DisconnectedSlot<C> {
        assert!(!self.is_connected());

        DisconnectedSlot(self)
    }

    /// Updates `Slot::connected` without notifying the `ConnectionSet`.
    ///
    /// Returns `Some(connected)` or `None` if this guard was already dropped.
    fn drop_without_notify(&mut self) -> Option<bool> {
        self.locked.take().map(|locked| {
            let connected = locked.is_some();
            self.slot.set_is_connected(connected);
            self.slot.locked.store(false, Ordering::Release);
            connected
        })
    }
}

const EXPECT_LOCKED: &str = "BUG: `SlotGuard::locked` should not be `None` in normal operation";
const EXPECT_CONNECTED: &str = "BUG: `ConnectedSlot` expects `Slot::connection` to be `Some`";

impl<C> ConnectedSlot<C> {
    pub fn take(mut self) -> (C, DisconnectedSlot<C>) {
        let conn = self.0.get_mut().take().expect(EXPECT_CONNECTED);
        (conn, self.0.assert_disconnected())
    }
}

impl<C> Deref for ConnectedSlot<C> {
    type Target = C;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.get().as_ref().expect(EXPECT_CONNECTED)
    }
}

impl<C> DerefMut for ConnectedSlot<C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut().as_mut().expect(EXPECT_CONNECTED)
    }
}

impl<C> DisconnectedSlot<C> {
    pub fn put(mut self, conn: C) -> ConnectedSlot<C> {
        *self.0.get_mut() = Some(conn);
        ConnectedSlot(self.0)
    }

    pub fn leak(mut self) {
        self.0.slot.leaked.store(true, Ordering::Release);
        self.0.drop_without_notify();
    }
}

impl<C> Drop for SlotGuard<C> {
    fn drop(&mut self) {
        let Some(connected) = self.drop_without_notify() else {
            return;
        };

        let event = if connected {
            &self.slot.global.unlock_event
        } else {
            &self.slot.global.disconnect_event
        };

        if event.notify(1.tag(self.slot.index).additional()) != 0 {
            return;
        }

        if connected {
            self.slot.unlock_event.notify(1);
            return;
        }

        if self.slot.disconnect_event.notify(1) != 0 {
            return;
        }

        if self.slot.global.num_connected() < self.slot.global.min_connections {
            self.slot.global.min_connections_event.notify(1);
        }
    }
}

impl Global {
    #[inline(always)]
    fn num_connected(&self) -> usize {
        self.num_connected.load(Ordering::Relaxed)
    }

    async fn listen(&self, pref: AcquirePreference) -> usize {
        match pref {
            AcquirePreference::Either => race(self.listen_unlocked(), self.listen_disconnected())
                .await
                .unwrap_or_else(|slot| slot),
            AcquirePreference::Connected => self.listen_unlocked().await,
            AcquirePreference::Disconnected => self.listen_disconnected().await,
        }
    }

    async fn listen_unlocked(&self) -> usize {
        listener!(self.unlock_event => listener);
        listener.await
    }

    async fn listen_disconnected(&self) -> usize {
        listener!(self.disconnect_event => listener);
        listener.await
    }
}

fn current_thread_id() -> usize {
    // FIXME: this can be replaced when this is stabilized:
    // https://doc.rust-lang.org/stable/std/thread/struct.ThreadId.html#method.as_u64
    static THREAD_ID: AtomicUsize = AtomicUsize::new(0);

    thread_local! {
        // `SeqCst` is possibly too strong since we don't need synchronization with
        // any other variable. I'm not confident enough in my understanding of atomics to be certain,
        // especially with regards to weakly ordered architectures.
        //
        // However, this is literally only done once on each thread, so it doesn't really matter.
        static CURRENT_THREAD_ID: usize = THREAD_ID.fetch_add(1, Ordering::SeqCst);
    }

    CURRENT_THREAD_ID.with(|i| *i)
}
