use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use crate::rt::TimeoutError;

pub async fn sleep(duration: Duration) {
    timeout_future(duration).await;
}

pub fn timeout<F: Future>(
    duration: Duration,
    future: F,
) -> impl Future<Output = Result<F::Output, TimeoutError>> {
    TimeoutFuture::new(future, timeout_future(duration))
}

fn timeout_future(duration: Duration) -> impl Future {
    async_io_global_executor::Timer::after(duration)
}

pub struct TimeoutFuture<F, D> {
    future: F,
    delay: D,
}

impl<F, D> TimeoutFuture<F, D> {
    fn new(future: F, delay: D) -> TimeoutFuture<F, D> {
        TimeoutFuture { future, delay }
    }
}

impl<F: Future, D: Future> Future for TimeoutFuture<F, D> {
    type Output = Result<F::Output, TimeoutError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future_polled = {
            let future = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.future) }; // safe, as self is Pin
            future.poll(cx)
        };
        match future_polled {
            Poll::Ready(v) => Poll::Ready(Ok(v)),
            Poll::Pending => {
                let delay = unsafe { self.map_unchecked_mut(|s| &mut s.delay) }; // safe, as self is Pin
                match delay.poll(cx) {
                    Poll::Ready(_) => Poll::Ready(Err(TimeoutError)),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}
