#[cfg(not(any(
    feature = "runtime-actix",
    feature = "runtime-async-std",
    feature = "runtime-tokio",
)))]
compile_error!(
    "one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features must be enabled"
);

#[cfg(any(
    all(feature = "runtime-actix", feature = "runtime-async-std"),
    all(feature = "runtime-actix", feature = "runtime-tokio"),
    all(feature = "runtime-async-std", feature = "runtime-tokio"),
))]
compile_error!(
    "only one of 'runtime-actix', 'runtime-async-std' or 'runtime-tokio' features can be enabled"
);

pub use native_tls::{self, Error as TlsError};

//
// Actix *OR* Tokio
//

#[cfg(all(
    any(feature = "runtime-tokio", feature = "runtime-actix"),
    not(feature = "runtime-async-std"),
))]
pub use tokio::{
    self, fs, io::AsyncRead, io::AsyncReadExt, io::AsyncWrite, io::AsyncWriteExt, net::TcpStream,
    task::spawn, task::yield_now, time::delay_for as sleep, time::timeout,
};

#[cfg(all(
    unix,
    any(feature = "runtime-tokio", feature = "runtime-actix"),
    not(feature = "runtime-async-std"),
))]
pub use tokio::net::UnixStream;

#[cfg(all(feature = "tokio-native-tls", not(feature = "async-native-tls")))]
pub use tokio_native_tls::{TlsConnector, TlsStream};

//
// tokio
//

#[cfg(all(
    feature = "runtime-tokio",
    not(any(feature = "runtime-actix", feature = "runtime-async-std",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::tokio::task::block_in_place(move || { $($expr)* })
    };
}

//
// actix
//

#[cfg(feature = "runtime-actix")]
pub use {actix_rt, actix_threadpool};

#[cfg(all(
    feature = "runtime-actix",
    not(any(feature = "runtime-tokio", feature = "runtime-async-std",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::actix_threadpool::run(move || { $($expr)* }).await.map_err(|err| match err {
            $crate::actix_threadpool::BlockingError::Error(e) => e,
            $crate::actix_threadpool::BlockingError::Canceled => panic!("{}", err)
        })
    };
}

//
// async-std
//

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio",))
))]
pub use async_std::{
    self, fs, future::timeout, io::prelude::ReadExt as AsyncReadExt,
    io::prelude::WriteExt as AsyncWriteExt, io::Read as AsyncRead, io::Write as AsyncWrite,
    net::TcpStream, task::sleep, task::spawn, task::yield_now,
};

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::async_std::task::spawn_blocking(move || { $($expr)* }).await
    };
}

#[cfg(all(
    unix,
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio",))
))]
pub use async_std::os::unix::net::UnixStream;

#[cfg(all(feature = "async-native-tls", not(feature = "tokio-native-tls")))]
pub use async_native_tls::{TlsConnector, TlsStream};

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio"))
))]
pub use async_std::task::block_on;

#[cfg(all(
    feature = "runtime-async-std",
    not(any(feature = "runtime-actix", feature = "runtime-tokio"))
))]
pub fn enter_runtime<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // no-op for async-std
    f()
}

/// Capture the last `.await` point in a backtrace.
///
/// NOTE: backtrace requires `.resolve()` still to get something that isn't all numbers.
#[cfg(all(
    feature = "runtime-async-std",
    feature = "capture-awaits",
    not(any(feature = "runtime-actix", feature = "runtime-tokio"))
))]
pub async fn capture_last_await<Fut>(fut: Fut) -> (Fut::Output, Option<backtrace::Backtrace>)
where
    Fut: futures::Future,
{
    use backtrace::Backtrace;
    use std::cell::Cell;

    async_std::task_local! {
        static LAST_AWAIT: Cell<Option<backtrace::Backtrace>> = Cell::new(None);
    }

    fn capture_await() {
        LAST_AWAIT.with(|last_await| last_await.set(Some(Backtrace::new_unresolved())));
    }

    LAST_AWAIT.with(|last| last.set(None));
    let res = capture_awaits(fut, capture_await).await;
    let last_await = LAST_AWAIT.with(|last_await| last_await.replace(None));
    (res, last_await)
}

#[cfg(all(
    any(feature = "runtime-tokio", feature = "runtime-actix"),
    not(feature = "runtime-async-std")
))]
pub use tokio_runtime::*;

#[cfg(any(feature = "runtime-tokio", feature = "runtime-actix"))]
mod tokio_runtime {
    use once_cell::sync::Lazy;
    use tokio::runtime::{self, Runtime};

    // lazily initialize a global runtime once for multiple invocations of the macros
    static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
        runtime::Builder::new()
            // `.basic_scheduler()` requires calling `Runtime::block_on()` which needs mutability
            .threaded_scheduler()
            .enable_io()
            .enable_time()
            .build()
            .expect("failed to initialize Tokio runtime")
    });

    pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
        RUNTIME.enter(|| RUNTIME.handle().block_on(future))
    }

    pub fn enter_runtime<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        RUNTIME.enter(f)
    }

    /// Capture the last `.await` point in a backtrace.
    ///
    /// NOTE: backtrace requires `.resolve()` still to get something that isn't all numbers.
    #[cfg(feature = "capture-awaits")]
    pub async fn capture_last_await<Fut>(fut: Fut) -> (Fut::Output, Option<backtrace::Backtrace>)
    where
        Fut: futures::Future,
    {
        use backtrace::Backtrace;
        use futures::{future::poll_fn, pin_mut, task::Context, Future};
        use std::cell::Cell;

        tokio::task_local!(
            static LAST_AWAIT: Cell<Option<Backtrace>>;
        );

        fn capture_await() {
            LAST_AWAIT.with(|last_await| last_await.set(Some(Backtrace::new_unresolved())))
        }

        LAST_AWAIT
            .scope(Cell::new(None), async move {
                let res = super::capture_awaits(fut, capture_await);
                let backtrace = LAST_AWAIT.with(|last_await| last_await.replace(None));

                (res, backtrace)
            })
            .await
    }
}

/// Create a `Waker` which captures a backtrace when it is cloned; when a `Waker` is cloned that
/// should mean that it's being squirreled away because the future it was called with is
/// going to return `Pending`.
#[cfg(feature = "capture-awaits")]
fn capture_await_waker(waker: &std::task::Waker, capture_await: fn()) -> std::task::Waker {
    use std::mem;
    use std::sync::Arc;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    struct WakerData {
        inner: std::task::Waker,
        capture_await: fn(),
    }

    // by requiring `capture_await` to be a regular fn pointer, we don't need to leak an
    // allocation for our VTable
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe fn clone(data: *const ()) -> RawWaker {
        // SAFETY: pointer must be the right type and Arc must not be dropped here without cloning.
        let data = Arc::from_raw(data as *const WakerData);
        (data.capture_await)();
        let cloned = data.clone();
        mem::forget(data);
        raw_waker(cloned)
    }

    unsafe fn wake(data: *const ()) {
        // SAFETY: pointer must be the right type
        // LEAK SAFETY: `Arc` *must* be dropped here
        let data = Arc::from_raw(data as *const WakerData);
        data.inner.wake_by_ref();
    }

    unsafe fn wake_by_ref(data: *const ()) {
        // SAFETY: pointer must be the right type and Arc must *NOT* be dropped here
        (data as *const WakerData)
            .as_ref()
            .unwrap()
            .inner
            .wake_by_ref();
    }

    unsafe fn drop(data: *const ()) {
        // SAFETY: pointer must be the right type
        // LEAK SAFETY: `Arc` *must* be dropped here
        let _ = Arc::from_raw(data as *const WakerData);
    }

    fn raw_waker(data: Arc<WakerData>) -> RawWaker {
        // SAFETY: Arc must not be dropped here; be sure to use Arc::into_raw
        RawWaker::new(Arc::into_raw(data) as *const (), &VTABLE)
    }

    unsafe {
        // SAFETY: verified above
        Waker::from_raw(raw_waker(Arc::new(WakerData {
            inner: waker.clone(),
            capture_await,
        })))
    }
}

#[cfg(feature = "capture-awaits")]
async fn capture_awaits<Fut>(fut: Fut, capture_await: fn()) -> Fut::Output
where
    Fut: futures::Future,
{
    use futures::{future::poll_fn, pin_mut, task::Context, Future};
    pin_mut!(fut);

    poll_fn(move |cx| {
        let waker = capture_await_waker(cx.waker(), capture_await);
        let mut cx = Context::from_waker(&waker);

        fut.as_mut().poll(&mut cx)
    })
    .await
}
