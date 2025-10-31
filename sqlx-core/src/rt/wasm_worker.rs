//! WASM-only single-threaded worker helpers for operations that touch wit-bindgen / wasip3.
//! These functions execute on the current-thread LocalSet so that `!Send` futures from
//! wit-bindgen never cross threads.

use futures::join;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_dispatch_simple_value() {
        async {
            let result = dispatch(|| async { 42 }).await;
            assert_eq!(result, 42);
        };
    }

    #[test]
    fn test_dispatch_string_value() {
        async {
            let result = dispatch(|| async { String::from("Hello from WASM worker!") }).await;
            assert_eq!(result, "Hello from WASM worker!");
        };
    }

    #[test]
    fn test_dispatch_computation() {
        async {
            let result = dispatch(|| async {
                let mut sum = 0;
                for i in 1..=100 {
                    sum += i;
                }
                sum
            })
            .await;
            assert_eq!(result, 5050);
        };
    }

    #[test]
    fn test_dispatch_with_sleep() {
        async {
            let start = std::time::Instant::now();

            let result = dispatch(|| async {
                crate::rt::sleep(Duration::from_millis(100)).await;
                "completed"
            })
            .await;

            let elapsed = start.elapsed();
            assert_eq!(result, "completed");
            assert!(elapsed >= Duration::from_millis(100));
        };
    }

    #[test]
    fn test_dispatch_multiple_sequential() {
        async {
            let result1 = dispatch(|| async { 10 }).await;
            let result2 = dispatch(|| async { 20 }).await;
            let result3 = dispatch(|| async { 30 }).await;

            assert_eq!(result1 + result2 + result3, 60);
        };
    }

    #[test]
    fn test_dispatch_multiple_concurrent() {
        async {
            let fut1 = dispatch(|| async { 1 });
            let fut2 = dispatch(|| async { 2 });
            let fut3 = dispatch(|| async { 3 });

            let (r1, r2, r3) = join!(fut1, fut2, fut3);
            assert_eq!(r1 + r2 + r3, 6);
        };
    }

    #[test]
    fn test_dispatch_with_closure_capture() {
        async {
            let multiplier = 5;

            let result = dispatch(move || async move { multiplier * 10 }).await;

            assert_eq!(result, 50);
        };
    }

    #[test]
    fn test_dispatch_with_option() {
        async {
            let result = dispatch(|| async { Some(42) }).await;

            assert_eq!(result, Some(42));
        };
    }

    #[test]
    fn test_dispatch_with_result_ok() {
        async {
            let result = dispatch(|| async { Ok::<i32, String>(100) }).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 100);
        };
    }

    #[test]
    fn test_dispatch_with_result_err() {
        async {
            let result =
                dispatch(|| async { Err::<i32, String>("error occurred".to_string()) }).await;

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "error occurred");
        };
    }

    #[test]
    fn test_dispatch_with_shared_state() {
        async {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            let result = dispatch(move || async move {
                counter_clone.fetch_add(10, Ordering::SeqCst);
                counter_clone.fetch_add(20, Ordering::SeqCst);
                counter_clone.load(Ordering::SeqCst)
            })
            .await;

            assert_eq!(result, 30);
            assert_eq!(counter.load(Ordering::SeqCst), 30);
        };
    }

    #[test]
    fn test_dispatch_multiple_with_shared_state() {
        async {
            let counter = Arc::new(AtomicU32::new(0));

            let mut handles = vec![];
            for i in 1..=5 {
                let counter_clone = counter.clone();
                let handle = tokio::spawn(async move {
                    dispatch(move || async move { counter_clone.fetch_add(i, Ordering::SeqCst) })
                        .await
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.await.unwrap();
            }

            // Sum of 1+2+3+4+5 = 15
            assert_eq!(counter.load(Ordering::SeqCst), 15);
        };
    }

    #[test]
    fn test_dispatch_with_boolean_flag() {
        async {
            let flag = Arc::new(AtomicBool::new(false));
            let flag_clone = flag.clone();

            dispatch(move || async move {
                flag_clone.store(true, Ordering::SeqCst);
            })
            .await;

            assert!(flag.load(Ordering::SeqCst));
        };
    }

    #[test]
    fn test_dispatch_nested_async_operations() {
        async {
            let result = dispatch(|| async {
                let inner_result = dispatch(|| async { 10 }).await;
                inner_result * 2
            })
            .await;

            assert_eq!(result, 20);
        };
    }

    #[test]
    fn test_dispatch_with_vec_result() {
        async {
            let result = dispatch(|| async { vec![1, 2, 3, 4, 5] }).await;

            assert_eq!(result.len(), 5);
            assert_eq!(result, vec![1, 2, 3, 4, 5]);
        };
    }

    #[test]
    fn test_dispatch_with_tuple_result() {
        async {
            let result = dispatch(|| async { (42, "hello", true) }).await;

            assert_eq!(result, (42, "hello", true));
        };
    }

    #[test]
    fn test_dispatch_with_struct_result() {
        #[derive(Debug, PartialEq)]
        struct TestData {
            id: u32,
            name: String,
        }

        async {
            let result = dispatch(|| async {
                TestData {
                    id: 1,
                    name: String::from("test"),
                }
            })
            .await;

            assert_eq!(result.id, 1);
            assert_eq!(result.name, "test");
        };
    }

    #[test]
    fn test_dispatch_long_running_job() {
        async {
            let start = std::time::Instant::now();

            let result = dispatch(|| async {
                // Simulate some work
                for _ in 0..5 {
                    crate::rt::sleep(Duration::from_millis(20)).await;
                }
                "long job completed"
            })
            .await;

            let elapsed = start.elapsed();
            assert_eq!(result, "long job completed");
            assert!(elapsed >= Duration::from_millis(100));
        };
    }

    #[test]
    fn test_dispatch_concurrent_long_jobs() {
        async {
            let start = std::time::Instant::now();

            let job1 = dispatch(|| async {
                crate::rt::sleep(Duration::from_millis(100)).await;
                1
            });

            let job2 = dispatch(|| async {
                crate::rt::sleep(Duration::from_millis(100)).await;
                2
            });

            let job3 = dispatch(|| async {
                crate::rt::sleep(Duration::from_millis(100)).await;
                3
            });

            let (r1, r2, r3) = join!(job1, job2, job3);
            let elapsed = start.elapsed();

            assert_eq!(r1 + r2 + r3, 6);
            // Should complete concurrently in ~100ms, not 300ms
            assert!(elapsed < Duration::from_millis(200));
        };
    }

    #[test]
    fn test_dispatch_with_conditional_logic() {
        async {
            let input = 5;

            let result = dispatch(move || async move {
                if input > 3 {
                    "greater"
                } else {
                    "lesser"
                }
            })
            .await;

            assert_eq!(result, "greater");
        };
    }

    #[test]
    fn test_dispatch_with_match_expression() {
        async {
            let value = 2;

            let result = dispatch(move || async move {
                match value {
                    1 => "one",
                    2 => "two",
                    3 => "three",
                    _ => "other",
                }
            })
            .await;

            assert_eq!(result, "two");
        };
    }

    #[test]
    fn test_dispatch_returns_unit() {
        async {
            let counter = Arc::new(AtomicU32::new(0));
            let counter_clone = counter.clone();

            dispatch(move || async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                // Implicitly returns ()
            })
            .await;

            assert_eq!(counter.load(Ordering::SeqCst), 1);
        };
    }

    #[test]
    fn test_dispatch_with_timeout() {
        async {
            let result = crate::rt::timeout(
                Duration::from_millis(500),
                dispatch(|| async {
                    crate::rt::sleep(Duration::from_millis(100)).await;
                    42
                }),
            )
            .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        };
    }

    #[test]
    fn test_dispatch_with_spawn() {
        async {
            let result = dispatch(|| async {
                let handle = crate::rt::spawn(async { 10 });
                let value = handle.await;
                value * 2
            })
            .await;

            assert_eq!(result, 20);
        };
    }

    #[test]
    fn test_dispatch_error_propagation() {
        async {
            let result: Result<i32, &str> = dispatch(|| async {
                // Simulate an operation that might fail
                if true {
                    Err("operation failed")
                } else {
                    Ok(42)
                }
            })
            .await;

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "operation failed");
        };
    }

    #[test]
    fn test_dispatch_chain_operations() {
        async {
            let step1 = dispatch(|| async { 5 }).await;
            let step2 = dispatch(move || async move { step1 * 2 }).await;
            let step3 = dispatch(move || async move { step2 + 10 }).await;

            assert_eq!(step3, 20);
        };
    }

    #[test]
    fn test_dispatch_with_large_data() {
        async {
            let result = dispatch(|| async {
                // Create a relatively large vector
                (0..1000).collect::<Vec<u32>>()
            })
            .await;

            assert_eq!(result.len(), 1000);
            assert_eq!(result[0], 0);
            assert_eq!(result[999], 999);
        };
    }
}
