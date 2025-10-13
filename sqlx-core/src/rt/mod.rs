use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use cfg_if::cfg_if;

#[cfg(feature = "_rt-async-io")]
pub mod rt_async_io;

#[cfg(feature = "_rt-tokio")]
pub mod rt_tokio;

#[cfg(target_arch = "wasm32")]
pub mod rt_wasip3;

#[derive(Debug, thiserror::Error)]
#[error("operation timed out")]
pub struct TimeoutError;

pub enum JoinHandle<T> {
    #[cfg(feature = "_rt-async-std")]
    AsyncStd(async_std::task::JoinHandle<T>),

    #[cfg(feature = "_rt-tokio")]
    Tokio(tokio::task::JoinHandle<T>),

    // WASI P3 runtime
    #[cfg(target_arch = "wasm32")]
    Wasip3(crate::rt::rt_wasip3::JoinHandle<T>),

    // Implementation shared by `smol` and `async-global-executor`
    #[cfg(feature = "_rt-async-task")]
    AsyncTask(Option<async_task::Task<T>>),

    // `PhantomData<T>` requires `T: Unpin`
    _Phantom(PhantomData<fn() -> T>),
}

pub async fn timeout<F: Future>(duration: Duration, f: F) -> Result<F::Output, TimeoutError> {
    #[cfg(all(feature = "_rt-tokio", target_arch = "wasm32"))]
    {
        let timeout_future = wasip3::clocks::monotonic_clock::wait_for(
            duration.as_nanos().try_into().unwrap_or(u64::MAX),
        );
        let mut timeout = core::pin::pin!(timeout_future);
        let mut f = core::pin::pin!(f);

        return core::future::poll_fn(|cx| match timeout.as_mut().poll(cx) {
            Poll::Ready(_) => Poll::Ready(Err(TimeoutError)),
            Poll::Pending => f.as_mut().poll(cx).map(Ok),
        })
        .await;
    }

    cfg_if! {
        if #[cfg(feature = "_rt-tokio")] {
            if rt_tokio::available() {
                tokio::time::timeout(duration, f)
                    .await
                    .map_err(|_| TimeoutError)
            } else {
                cfg_if! {
                    if #[cfg(feature = "_rt-async-io")] {
                        rt_async_io::timeout(duration, f).await
                    } else {
                        missing_rt((duration, f))
                    }
                }
            }
        } else if #[cfg(feature = "_rt-async-io")] {
            rt_async_io::timeout(duration, f).await
        } else {
            missing_rt((duration, f))
        }
    }
}

pub async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    {
        wasip3::clocks::monotonic_clock::wait_for(
            duration.as_nanos().try_into().unwrap_or(u64::MAX),
        )
        .await;
        return;
    }

    cfg_if! {
        if #[cfg(feature = "_rt-tokio")] {
            if rt_tokio::available() {
                tokio::time::sleep(duration).await
            } else {
                cfg_if! {
                    if #[cfg(feature = "_rt-async-io")] {
                        rt_async_io::sleep(duration).await
                    } else {
                        #[cfg(not(any(feature = "_rt-async-std", target_arch = "wasm32")))]
                        missing_rt(duration)
                    }
                }
            }
        } else if #[cfg(feature = "_rt-async-io")] {
            rt_async_io::sleep(duration).await
        } else {
            #[cfg(not(any(feature = "_rt-async-std", target_arch = "wasm32")))]
            missing_rt(duration)
        }
    }
}

#[track_caller]
pub fn spawn<F>(fut: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(all(feature = "_rt-tokio", target_arch = "wasm32"))]
    {
        return JoinHandle::Wasip3(crate::rt::rt_wasip3::spawn(fut));
    }

    #[cfg(feature = "_rt-tokio")]
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return JoinHandle::Tokio(handle.spawn(fut));
    }

    cfg_if! {
        if #[cfg(feature = "_rt-async-global-executor")] {
            JoinHandle::AsyncTask(Some(async_global_executor::spawn(fut)))
        } else if #[cfg(feature = "_rt-smol")] {
            JoinHandle::AsyncTask(Some(smol::spawn(fut)))
        } else if #[cfg(feature = "_rt-async-std")] {
            JoinHandle::AsyncStd(async_std::task::spawn(fut))
        } else {
            missing_rt(fut)
        }
    }
}

#[cfg(all(feature = "_rt-tokio", target_arch = "wasm32"))]
#[track_caller]
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    JoinHandle::Wasip3(crate::rt::rt_wasip3::spawn(
        crate::rt::rt_wasip3::spawn_blocking(f),
    ))
}

#[cfg(not(target_arch = "wasm32"))]
#[track_caller]
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    #[cfg(feature = "_rt-tokio")]
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return JoinHandle::Tokio(handle.spawn_blocking(f));
    }

    cfg_if! {
        if #[cfg(feature = "_rt-async-global-executor")] {
            JoinHandle::AsyncTask(Some(async_global_executor::spawn_blocking(f)))
        } else if #[cfg(feature = "_rt-smol")] {
            JoinHandle::AsyncTask(Some(smol::unblock(f)))
        } else if #[cfg(feature = "_rt-async-std")] {
            JoinHandle::AsyncStd(async_std::task::spawn_blocking(f))
        } else {
            missing_rt(f)
        }
    }
}

pub async fn yield_now() {
    #[cfg(all(feature = "_rt-tokio", target_arch = "wasm32"))]
    {
        return crate::rt::rt_wasip3::yield_now().await;
    }

    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        return tokio::task::yield_now().await;
    }

    // `smol`, `async-global-executor`, and `async-std` all have the same implementation for this.
    //
    // By immediately signaling the waker and then returning `Pending`,
    // this essentially just moves the task to the back of the runnable queue.
    //
    // There isn't any special integration with the runtime, so we can save code by rolling our own.
    //
    // (Tokio's implementation is nearly identical too,
    //  but has additional integration with `tracing` which may be useful for debugging.)
    let mut yielded = false;

    std::future::poll_fn(|cx| {
        if !yielded {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    })
    .await
}

#[cfg(not(target_arch = "wasm32"))]
#[track_caller]
pub fn test_block_on<F: Future>(f: F) -> F::Output {
    #[cfg(feature = "_rt-async-io")]
    {
        return async_io::block_on(f);
    }

    #[cfg(feature = "_rt-tokio")]
    {
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to start Tokio runtime")
            .block_on(f);
    }

    #[cfg(all(
        feature = "_rt-async-std",
        not(feature = "_rt-async-io"),
        not(feature = "_rt-tokio")
    ))]
    {
        return async_std::task::block_on(f);
    }

    #[cfg(not(any(
        feature = "_rt-async-io",
        feature = "_rt-async-std",
        feature = "_rt-tokio"
    )))]
    {
        missing_rt(f)
    }
}

#[cfg(target_arch = "wasm32")]
#[track_caller]
pub fn test_block_on<F: Future + 'static>(f: F) -> F::Output
where
    F::Output: 'static,
{
    wasip3::wit_bindgen::rt::async_support::block_on(f)
}

#[track_caller]
pub const fn missing_rt<T>(_unused: T) -> ! {
    if cfg!(feature = "_rt-tokio") {
        panic!("this functionality requires a Tokio context")
    }

    panic!("one of the `runtime` features of SQLx must be enabled")
}

impl<T: Send + 'static> Future for JoinHandle<T> {
    type Output = T;

    #[track_caller]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut *self {
            #[cfg(feature = "_rt-async-std")]
            Self::AsyncStd(handle) => Pin::new(handle).poll(cx),

            #[cfg(feature = "_rt-async-task")]
            Self::AsyncTask(task) => Pin::new(task)
                .as_pin_mut()
                .expect("BUG: task taken")
                .poll(cx),

            #[cfg(feature = "_rt-tokio")]
            Self::Tokio(handle) => Pin::new(handle)
                .poll(cx)
                .map(|res| res.expect("spawned task panicked")),

            #[cfg(target_arch = "wasm32")]
            Self::Wasip3(handle) => Pin::new(handle).poll(cx),

            Self::_Phantom(_) => {
                let _ = cx;
                unreachable!("runtime should have been checked on spawn")
            }
        }
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        match self {
            // `async_task` cancels on-drop by default.
            // We need to explicitly detach to match Tokio and `async-std`.
            #[cfg(feature = "_rt-async-task")]
            Self::AsyncTask(task) => {
                if let Some(task) = task.take() {
                    task.detach();
                }
            }
            _ => (),
        }
    }
}
