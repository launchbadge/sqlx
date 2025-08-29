use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use cfg_if::cfg_if;

#[cfg(feature = "_rt-async-io")]
pub mod rt_async_io;

#[cfg(feature = "_rt-async-global-executor")]
pub mod rt_async_global_executor;

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
    #[cfg(debug_assertions)]
    let f = Box::pin(f);

    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        #[allow(clippy::needless_return)]
        return tokio::time::timeout(duration, f)
            .await
            .map_err(|_| TimeoutError);
    }

    cfg_if! {
        if #[cfg(feature = "_rt-async-io")] {
            rt_async_io::timeout(duration, f).await
        } else {
            missing_rt((duration, f))
        }
    }
}

pub async fn sleep(duration: Duration) {
    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        return tokio::time::sleep(duration).await;
    }

    cfg_if! {
        if #[cfg(feature = "_rt-async-io")] {
            rt_async_io::sleep(duration).await
        } else {
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
    #[cfg(feature = "_rt-tokio")]
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return JoinHandle::Tokio(handle.spawn(fut));
    }

    cfg_if! {
        if #[cfg(feature = "_rt-async-global-executor")] {
            JoinHandle::AsyncGlobalExecutor(rt_async_global_executor::JoinHandle {
                task: Some(async_global_executor::spawn(fut)),
            })
        } else if #[cfg(feature = "_rt-async-std")] {
            JoinHandle::AsyncStd(async_std::task::spawn(fut))
        } else if #[cfg(feature = "_rt-smol")] {
            JoinHandle::Smol(rt_smol::JoinHandle {
                task: Some(smol::spawn(fut)),
            })
        } else {
            missing_rt(fut)
        }
    }
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

    cfg_if! {
        if #[cfg(feature = "_rt-async-global-executor")] {
            JoinHandle::AsyncGlobalExecutor(rt_async_global_executor::JoinHandle {
                task: Some(async_global_executor::spawn_blocking(f)),
            })
        } else if #[cfg(feature = "_rt-async-std")] {
            JoinHandle::AsyncStd(async_std::task::spawn_blocking(f))
        } else if #[cfg(feature = "_rt-smol")] {
            JoinHandle::Smol(rt_smol::JoinHandle {
                task: Some(smol::unblock(f)),
            })
        } else {
            missing_rt(f)
        }
    }
}

pub async fn yield_now() {
    #[cfg(feature = "_rt-tokio")]
    if rt_tokio::available() {
        return tokio::task::yield_now().await;
    }

    cfg_if! {
        if #[cfg(feature = "_rt-async-global-executor")] {
            rt_async_global_executor::yield_now().await
        } else if #[cfg(feature = "_rt-async-std")] {
            async_std::task::yield_now().await
        } else if #[cfg(feature = "_rt-smol")] {
            smol::future::yield_now().await
        } else {
            missing_rt(())
        }
    }
}

#[track_caller]
pub fn test_block_on<F: Future>(f: F) -> F::Output {
    cfg_if! {
        if #[cfg(feature = "_rt-async-io")] {
            async_io::block_on(f)
        } else if #[cfg(feature = "_rt-tokio")] {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to start Tokio runtime")
                .block_on(f)
        } else {
            missing_rt(f)
        }
    }
}

#[track_caller]
pub const fn missing_rt<T>(_unused: T) -> ! {
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
