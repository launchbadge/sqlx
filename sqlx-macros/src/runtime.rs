#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(feature = "runtime-async-std")]
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    // builds a runtime, but only for use at compile time, not app runtime?
    smol::run(future)
}

#[cfg(feature = "runtime-async-std")]
pub(crate) mod fs {
    use std::fs;
    use std::path::Path;

    // Only need read_to_string
    pub async fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
        let path = path.as_ref().to_owned();
        smol::Task::blocking(async move { fs::read_to_string(&path) }).await
    }

#[cfg(feature = "runtime-tokio")]
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
