//! WASM-only single-threaded worker helpers for operations that touch wit-bindgen / wasip3.
//! These functions execute on the current-thread LocalSet so that `!Send` futures from
//! wit-bindgen never cross threads.

use log::debug;
use wasip3::wit_bindgen::rt::async_support;
use wasip3::wit_bindgen::rt::async_support::futures::channel::oneshot;

/// Dispatch a job to run on the wasip3/local (single-threaded) runtime and
/// return the result across a Send-capable oneshot receiver. The provided
/// closure `job` is executed inside the spawned wasip3 task and may contain
/// `!Send` futures (e.g. from wit-bindgen). The returned future (awaiting the
/// oneshot) is Send so callers that require Send can await it.
pub async fn dispatch<R, Fut, F>(job: F) -> R
where
    F: FnOnce() -> Fut + 'static,
    Fut: core::future::Future<Output = R> + 'static,
    R: Send + 'static,
{
    let (tx, rx) = oneshot::channel::<R>();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    debug!("wasm_worker: dispatch job at {}ms", now);

    async_support::spawn(async move {
        // Yield to the wasip3 scheduler so any tasks spawned by `job()` get
        // an opportunity to be polled quickly.
        async_support::yield_async().await;

        let res = job().await;
        let _ = tx.send(res);
    });
    let out = rx.await.expect("wasip3 task canceled");
    out
}
