// see `SAFETY:` annotations
#![allow(unsafe_code)]

use parking_lot::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard};
use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use std::thread::{self, Thread};
use std::time::Instant;

/// An intrusive list of waiting tasks.
///
/// Tasks wait by calling `.wait().await` for async code or `.wait().block_on(deadline)`
/// for blocking code where `deadline` is `Option<Instant>`
pub struct WaitList(RwLock<ListInner>);

struct ListInner {
    // NOTE: these must either both be null or both be pointing to a node
    /// The head of the list; if NULL then the list is empty.
    head: *mut Node,
    /// The tail of the list; if NULL then the list is empty.
    tail: *mut Node,
}

// SAFETY: access to `Node` pointers must be protected by a lock
// this could potentially be made lock-free but the critical sections are short
// so using a lightweight RwLock like from `parking_lot` seemed reasonable
unsafe impl Send for ListInner {}
unsafe impl Sync for ListInner {}

impl WaitList {
    pub fn new() -> Self {
        WaitList(RwLock::new(ListInner { head: ptr::null_mut(), tail: ptr::null_mut() }))
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.0.read();
        inner.head.is_null() && inner.tail.is_null()
    }

    pub fn wake_one(&self) {
        self.0.read().wake(false)
    }

    pub fn wake_all(&self) {
        self.0.read().wake(true)
    }

    /// Wait in this waitlist for a call to either `.wake_one()` or `.wake_all()`.
    ///
    /// The returned handle may either be `.await`ed for async code, or you can call
    /// `.block_on(deadline)` for blocking code, where `deadline` is the optional `Instant`
    /// at which to stop waiting.
    pub fn wait(&self) -> Wait<'_> {
        Wait { list: &self.0, node: None, actually_woken: bool, _not_unpin: PhantomPinned }
    }
}

impl ListInner {
    /// Wake either one or all nodes in the list.
    fn wake(&self, all: bool) {
        let mut node_p: *const Node = inner.head;

        // SAFETY: `node_p` is not dangling as long as we have at least a shared lock
        // (implied by having `&self`)
        while let Some(node) = unsafe { node_p.as_ref() } {
            // `.wake()` only returns `true` if the node was not already woken
            if node.wake() && !all {
                break;
            }

            node_p = node.next;
        }
    }
}

pub struct Wait<'a> {
    list: &'a RwLock<ListInner>,
    /// SAFETY: `Node` must not be modified without a lock
    /// SAFETY: `Node` may not be moved once it's entered in the list
    node: Option<Node>,
    actually_woken: bool,
    _not_unpin: PhantomPinned,
}

/// cancel-safe
impl<'a> Future for Wait<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let node = self.get_node(|| Wake::Waker(cx.waker().clone()));

        if node.woken.load(Ordering::Acquire) {
            // SAFETY: not moving out of `self` here
            unsafe { self.get_unchecked_mut().actually_woken = true }
            Poll::Ready(())
        } else {
            let wake = RwLock::upgradable_read(&node.wake);

            // make sure our `Waker` is up to date;
            // the waker may change if the task moves between threads
            if !wake.waker_eq(cx.waker()) {
                *RwLockUpgradableReadGuard::upgrade(wake) = Wake::Waker(cx.waker().clone());
            }

            Poll::Pending
        }
    }
}

impl<'a> Wait<'a> {
    /// Insert a node into the parent `WaitList` referred to by `self` and return it.
    ///
    /// The provided closure should return the appropriate `Wake` variant for waking the calling
    /// task.
    fn get_node(self: Pin<&mut Self>, get_wake: impl FnOnce() -> Wake) -> &Node {
        // SAFETY: `this.node` must not be moved once it's entered in the list
        let this = unsafe { self.get_unchecked_mut() };

        if let Some(ref node) = this.node {
            node
        } else {
            // FIXME: use `Option::insert()` when stable
            let node = this.node.get_or_insert_with(|| Node::new(get_wake()));

            // SAFETY: we need an exclusive lock to modify the list
            let mut list = this.list.write();

            if list.head.is_null() {
                // sanity check; see `ListInner` definition
                assert!(list.tail.is_null());

                // the list is empty so insert this node as both the head and tail
                list.head = node;
                list.tail = node;
            } else {
                // sanity check; see `ListInner` definition
                assert!(!list.tail.is_null());

                // the list is nonempty so insert this node as the tail

                // SAFETY: `list.tail` is not null because of the above assert and
                // not dangling as long as we have an exclusive lock for modifying the list
                // (or any nodes in it)
                unsafe {
                    // set the `next` pointer of the previous tail to this node
                    (*list.tail).next = node;
                }
                node.prev = list.tail;
                list.tail = node;
            }

            node
        }
    }

    /// Block until woken.
    ///
    /// Returns `true` if we were woken without the deadline elapsing, `false` if the deadline elapsed.
    /// If no deadline is set then this always returns `true` but *will block* until woken.
    #[cfg(feature = "blocking")]
    pub fn block_on(mut self, deadline: Option<Instant>) -> bool {
        // SAFETY:`self.node` may not be moved once entered in the list (`.get_node()` is called)
        let mut this = unsafe { Pin::new_unchecked(&mut self) };
        let node = this.as_mut().get_node(|| Wake::Thread(thread::current()));

        while !node.woken.load(Ordering::Acquire) {
            if let Some(deadline) = deadline {
                let now = Instant::now();

                if deadline < now {
                    return false;
                } else {
                    // N.B. may wake spuriously
                    thread::park_timeout(deadline - now);
                }
            } else {
                // N.B. may return spuriously
                thread::park();
            }
        }

        // SAFETY: we're not moving out of `this` here
        unsafe {
            this.get_unchecked_mut().actually_woken = true;
        }

        true
    }
}

// SAFETY: since futures must be pinned to be polled we can be sure that `Drop::drop()` is called
// because there's no way to leak a future without the memory location remaining valid for the
// life of the program:
// * can't be moved into `mem::forget()` or an Rc-cycle because it's pinned
// * leaking `Pin<Box<Wait>>` or via Rc-cycle keeps it around forever, perfectly fine
// * aborting the program means it's not our problem anymore
//
// The only way this could cause memory issues is if the *thread* is aborted without unwinding
// or aborting the process, which doesn't have a safe API in Rust and the C APIs for canceling
// threads don't recommend doing it either for similar reasons.
// * https://man7.org/linux/man-pages/man3/pthread_exit.3.html#DESCRIPTION
// * https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-exitthread#remarks
//
// However, if Rust were to gain a safe API for instantly exiting a thread it would completely break
// the assumptions that the `Pin` API are built on so it's not something for us to worry about
// specifically.
impl<'a> Drop for Wait<'a> {
    fn drop(&mut self) {
        if let Some(node) = &self.node {
            // if we were inserted into the list then remove the node from the list,
            // linking the previous node (if applicable) to the next node (if applicable)

            // SAFETY: we must have an exclusive lock while we're futzing with the list
            let mut list = self.list.write();

            // SAFETY: `prev` cannot be dangling while we have an exclusive lock
            if let Some(prev) = unsafe { node.prev.as_mut() } {
                // set the `next` pointer of the previous node to this node's `next` pointer
                // note: `node.next` may be null which means we're the tail of the list
                prev.next = node.next;
            } else {
                // we were the head of the list so we set the head to the next node
                list.head = node.next;
            }

            // SAFETY: `next` cannot be dangling while we have an exclusive lock
            if let Some(next) = unsafe { node.next.as_mut() } {
                // set the `prev` pointer of the next node to this node's `prev` pointer
                // note: `node.prev` may be null which means we're the head of the list
                next.prev = node.prev;
            } else {
                // we were the tail of the list so we set the tail to the previous node
                list.tail = node.prev;
            }

            // sanity check; see `ListInner` definition
            assert_eq!(list.head.is_null(), list.tail.is_null());

            // if this node was marked woken but we didn't actually wake,
            // then we need to wake the next node in the list
            if node.woken.load(Ordering::Acquire) && !self.actually_woken {
                // we don't need an exclusive lock anymore
                RwLockWriteGuard::downgrade(list).wake(false);
            }
        }
    }
}

struct Node {
    /// The previous node in the list. If NULL, then this node is the head of the list.
    prev: *mut Node,
    /// The next node in the list. If NULL, then this node is the tail of the list.
    next: *mut Node,
    woken: AtomicBool,
    wake: RwLock<Wake>,
}

// SAFETY: access to `Node` pointers must be protected by a lock
unsafe impl Send for Node {}
unsafe impl Sync for Node {}

impl Node {
    fn new(wake: Wake) -> Self {
        Node {
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            woken: AtomicBool::new(false),
            wake: RwLock::new(wake),
        }
    }

    /// Returns `true` if this node was woken by this call, `false` otherwise.
    fn wake(&self) -> bool {
        let do_wake =
            self.woken.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok();

        if do_wake {
            match &*self.wake.read() {
                Wake::Waker(waker) => waker.wake_by_ref(),
                #[cfg(feature = "blocking")]
                Wake::Thread(thread) => thread.unpark(),
            }
        }

        do_wake
    }
}

enum Wake {
    Waker(Waker),
    #[cfg(feature = "blocking")]
    Thread(Thread),
}

impl Wake {
    fn waker_eq(&self, waker: &Waker) -> bool {
        match self {
            Self::Waker(waker_) => waker_.will_wake(waker),
            #[cfg(feature = "blocking")]
            _ => false,
        }
    }
}

// note: this test should take about 2 minutes to run!
#[test]
#[cfg(feature = "blocking")]
fn test_wait_list_blocking() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    const NUM_THREADS: u64 = 200;

    let list = Arc::new(WaitList::new());
    let mut threads = Vec::new();

    // create an arbitrary pattern of deadlines; some of these may elapse, others may not
    // the ultimate goal of this test is to make sure that no threads _deadlock_ or segfault
    for i in 1..NUM_THREADS {
        let ms = i + i * 25 % 100;

        let deadline = (i < 100).then(|| Instant::now() + Duration::from_millis(ms));

        let list = Arc::new(list.clone());
        let thread = Arc::new(AtomicBool::new(false));

        threads.push((thread.clone(), deadline));

        thread::spawn(move || {
            list.wait().block_on(deadline);
            thread.store(true, Ordering::Release);
        });
    }

    //
    for _ in 1..NUM_THREADS {
        thread::sleep(Duration::from_millis(5));
        list.wake_one();
    }

    // wait enough time for all timeouts to elapse
    thread::sleep(Duration::from_secs(60));

    for (i, (thread, deadline)) in threads.iter().enumerate() {
        assert!(
            thread.load(Ordering::Acquire),
            "thread {} did not exit; deadline: {:?}",
            i,
            deadline
        );
    }
}

// #[cfg(all(test, feature = "async"))]
// mod test_async {
//     use super::WaitList;
//
//     #[cfg(feature = "tokio")]
//
//     async fn test_waiter_list() {
//         use futures::future::{join_all, FutureExt};
//         use futures::pin_mut;
//         use std::sync::Arc;
//         use std::time::Duration;
//
//         let list = Arc::new(WaitList::new());
//         let mut tasks = Vec::new();
//
//         for _ in 0..1000 {
//             let list = list.clone();
//
//             tasks.push(spawn(async move {
//                 list.wait().await;
//
//                 list.wait().await;
//             }));
//         }
//
//         let waker = async {
//             loop {
//                 list.wake_one();
//                 yield_now().await;
//             }
//         }
//         .fuse();
//
//         let timeout = timeout(Duration::from_secs(10), join_all(tasks)).fuse();
//
//         pin_mut!(waker);
//         pin_mut!(timeout);
//
//         futures::select_biased!(
//             res = timeout => res.expect("all tasks should have exited by now"),
//             _ = waker => unreachable!("waker shouldn't have quit"),
//         );
//     }
// }
//
// // N.B. test will run forever
// #[test]
// #[ignore]
// fn test_waiter_list_forever() {
//     use async_std::{
//         future::{timeout, Future},
//         task,
//     };
//     use futures::future::poll_fn;
//     use futures::pin_mut;
//     use futures::stream::{FuturesUnordered, StreamExt};
//     use std::sync::Arc;
//     use std::thread;
//     use std::time::Duration;
//
//     let list = Arc::new(WaitList::new());
//
//     let list_ = list.clone();
//     task::spawn(async move {
//         let mut unordered = FuturesUnordered::new();
//
//         loop {
//             unordered.push(WaitList::wait(&list_));
//             let _ = timeout(Duration::from_millis(50), unordered.next()).await;
//         }
//     });
//
//     let list_ = list.clone();
//     task::spawn(poll_fn::<(), _>(move |cx| {
//         let yielder = task::yield_now();
//         pin_mut!(yielder);
//         let _ = yielder.poll(cx);
//
//         let park = WaitList::wait(&list_);
//         pin_mut!(park);
//         let _ = park.poll(cx);
//
//         Poll::Pending
//     }));
//
//     for num in (0..5).cycle() {
//         for _ in 0..num {
//             list.wake_one();
//         }
//
//         thread::sleep(Duration::from_millis(50));
//     }
// }
