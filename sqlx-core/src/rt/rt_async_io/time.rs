use std::{
    future::Future,
    pin::pin,
    time::{Duration, Instant},
};

use futures_util::future::{select, Either};

use crate::rt::TimeoutError;

pub async fn sleep(duration: Duration) {
    async_io::Timer::after(duration).await;
}

pub async fn sleep_until(deadline: Instant) {
    async_io::Timer::at(deadline).await;
}

pub async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, TimeoutError> {
    match select(pin!(future), pin!(sleep(duration))).await {
        Either::Left((result, _)) => Ok(result),
        Either::Right(_) => Err(TimeoutError),
    }
}
