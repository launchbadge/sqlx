//! WASM-only single-threaded worker helpers for operations that touch wit-bindgen / wasip3.
//! These functions execute on the current-thread LocalSet so that `!Send` futures from
//! wit-bindgen never cross threads.

use async_lock::Mutex;
use once_cell::sync::OnceCell;
use wasip3::wit_bindgen::rt::async_support::futures::channel::oneshot;

use crate::net::SocketIntoBox;
use crate::Result as SqlxResult;
use wasip3::wit_bindgen::rt::async_support;

// A simple mutex to serialize WASI operations on the current thread/local runtime.
// We use OnceCell so the mutex is initialized lazily.
static WASM_WORKER_LOCK: OnceCell<Mutex<()>> = OnceCell::new();

fn worker_lock() -> &'static Mutex<()> {
    WASM_WORKER_LOCK.get_or_init(|| Mutex::new(()))
}

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

    // Spawn the job into the wasip3 async runtime. We hold the worker mutex
    // during the job to serialize access to any non-thread-safe wasi bindings.
    eprintln!("wasm_worker: dispatch job");
    async_support::spawn(async move {
        let _guard = worker_lock().lock().await;
        eprintln!("wasm_worker: acquired lock, running job");
        let res = job().await;
        eprintln!("wasm_worker: job completed, sending result");
        let _ = tx.send(res);
    });

    eprintln!("wasm_worker: awaiting job result");
    // Await the result from the spawned task. The receiver is Send.
    rx.await.expect("wasip3 task canceled")
}
