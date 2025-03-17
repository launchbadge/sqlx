use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub fn yield_now() -> impl Future<Output = ()> {
    YieldNow(false)
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    // The futures executor is implemented as a FIFO queue, so all this future
    // does is re-schedule the future back to the end of the queue, giving room
    // for other futures to progress.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
