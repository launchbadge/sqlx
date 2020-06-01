// NOTE: this is separate from sqlx-rt because of the non-production nature of it

#[cfg(feature = "runtime-async-std")]
pub(crate) use async_std::task::block_on;

#[cfg(any(feature = "runtime-tokio", feature = "runtime-actix"))]
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
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

    RUNTIME.enter(|| futures::executor::block_on(future))
}
