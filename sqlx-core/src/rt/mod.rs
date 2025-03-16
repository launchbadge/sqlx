use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

#[cfg(feature = "_rt-async-global-executor")]
pub mod rt_async_global_executor;

#[cfg(feature = "_rt-async-std")]
pub mod rt_async_std;

#[cfg(feature = "_rt-smol")]
pub mod rt_smol;

#[cfg(feature = "_rt-tokio")]
pub mod rt_tokio;

#[derive(Debug, thiserror::Error)]
#[error("operation timed out")]
pub struct TimeoutError;

pub enum JoinHandle<T> {
    #[cfg(feature = "_rt-async-global-executor")]
    AsyncGlobalExecutor(rt_async_global_executor::JoinHandle<T>),
    #[cfg(feature = "_rt-async-std")]
    AsyncStd(async_std::task::JoinHandle<T>),
    #[cfg(feature = "_rt-smol")]
    Smol(rt_smol::JoinHandle<T>),
    #[cfg(feature = "_rt-tokio")]
    Tokio(tokio::task::JoinHandle<T>),
    // `PhantomData<T>` requires `T: Unpin`
    _Phantom(PhantomData<fn() -> T>),
}

pub async fn timeout<F: Future>(duration: Duration, f: F) -> Result<F::Output, TimeoutError> {
    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        return tokio::time::timeout(duration, f)
            .await
            .map_err(|_| TimeoutError);
    }

    #[cfg(feature = "_rt-async-global-executor")]
    {
        return rt_async_global_executor::timeout(duration, f).await;
    }

    #[cfg(feature = "_rt-smol")]
    {
        return rt_smol::timeout(duration, f).await;
    }

    #[cfg(feature = "_rt-async-std")]
    {
        return async_std::future::timeout(duration, f)
            .await
            .map_err(|_| TimeoutError);
    }

    #[cfg(not(all(
        feature = "_rt-async-global-executor",
        feature = "_rt-async-std",
        feature = "_rt-smol"
    )))]
    #[allow(unreachable_code)]
    missing_rt((duration, f))
}

pub async fn sleep(duration: Duration) {
    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        return tokio::time::sleep(duration).await;
    }

    #[cfg(feature = "_rt-async-global-executor")]
    {
        return rt_async_global_executor::sleep(duration).await;
    }

    #[cfg(feature = "_rt-smol")]
    {
        return rt_smol::sleep(duration).await;
    }

    #[cfg(feature = "_rt-async-std")]
    {
        return async_std::task::sleep(duration).await;
    }

    #[cfg(not(all(
        feature = "_rt-async-global-executor",
        feature = "_rt-async-std",
        feature = "_rt-smol"
    )))]
    #[allow(unreachable_code)]
    missing_rt(duration)
}

#[track_caller]
pub fn spawn<F>(fut: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(feature = "_rt-tokio")]
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return JoinHandle::Tokio(handle.spawn(fut));
    }

    #[cfg(feature = "_rt-async-global-executor")]
    {
        return JoinHandle::AsyncGlobalExecutor(rt_async_global_executor::JoinHandle {
            task: Some(async_global_executor::spawn(fut)),
        });
    }

    #[cfg(feature = "_rt-async-std")]
    {
        return JoinHandle::AsyncStd(async_std::task::spawn(fut));
    }

    #[cfg(not(all(
        feature = "_rt-async-global-executor",
        feature = "_rt-async-std",
        feature = "_rt-smol"
    )))]
    #[allow(unreachable_code)]
    missing_rt(fut)
}

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

    #[cfg(feature = "_rt-async-global-executor")]
    {
        return JoinHandle::AsyncGlobalExecutor(rt_async_global_executor::JoinHandle {
            task: Some(async_global_executor::spawn_blocking(f)),
        });
    }

    #[cfg(feature = "_rt-async-std")]
    {
        return JoinHandle::AsyncStd(async_std::task::spawn_blocking(f));
    }

    #[cfg(feature = "_rt-smol")]
    {
        return JoinHandle::Smol(rt_smol::JoinHandle {
            task: Some(smol::unblock(f)),
        });
    }

    #[cfg(not(all(
        feature = "_rt-async-global-executor",
        feature = "_rt-async-std",
        feature = "_rt-smol"
    )))]
    #[allow(unreachable_code)]
    missing_rt(f)
}

pub async fn yield_now() {
    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        return tokio::task::yield_now().await;
    }

    #[cfg(feature = "_rt-async-global-executor")]
    {
        return rt_async_global_executor::yield_now().await;
    }

    #[cfg(feature = "_rt-async-std")]
    {
        return async_std::task::yield_now().await;
    }

    #[cfg(feature = "_rt-smol")]
    {
        return smol::future::yield_now().await;
    }

    #[cfg(not(all(
        feature = "_rt-async-global-executor",
        feature = "_rt-async-std",
        feature = "_rt-smol"
    )))]
    #[allow(unreachable_code)]
    missing_rt(())
}

#[track_caller]
pub fn test_block_on<F: Future>(f: F) -> F::Output {
    #[cfg(feature = "_rt-tokio")]
    {
        if rt_tokio::available() {
            return tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to start Tokio runtime")
                .block_on(f);
        }
    }

    #[cfg(feature = "_rt-async-global-executor")]
    {
        return async_io_global_executor::block_on(f);
    }

    #[cfg(feature = "_rt-async-std")]
    {
        return async_std::task::block_on(f);
    }

    #[cfg(feature = "_rt-smol")]
    {
        return smol::block_on(f);
    }

    #[cfg(not(all(
        feature = "_rt-async-global-executor",
        feature = "_rt-async-std",
        feature = "_rt-smol"
    )))]
    #[allow(unreachable_code)]
    missing_rt(f)
}

#[track_caller]
pub fn missing_rt<T>(_unused: T) -> ! {
    if cfg!(feature = "_rt-tokio") {
        panic!("this functionality requires a Tokio context")
    }

    panic!("one of the `runtime-async-global-executor`, `runtime-async-std`, `runtime-smol`, or `runtime-tokio` feature must be enabled")
}

impl<T: Send + 'static> Future for JoinHandle<T> {
    type Output = T;

    #[track_caller]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut *self {
            #[cfg(feature = "_rt-async-global-executor")]
            Self::AsyncGlobalExecutor(handle) => Pin::new(handle).poll(cx),
            #[cfg(feature = "_rt-async-std")]
            Self::AsyncStd(handle) => Pin::new(handle).poll(cx),
            #[cfg(feature = "_rt-smol")]
            Self::Smol(handle) => Pin::new(handle).poll(cx),
            #[cfg(feature = "_rt-tokio")]
            Self::Tokio(handle) => Pin::new(handle)
                .poll(cx)
                .map(|res| res.expect("spawned task panicked")),
            Self::_Phantom(_) => {
                let _ = cx;
                unreachable!("runtime should have been checked on spawn")
            }
        }
    }
}
