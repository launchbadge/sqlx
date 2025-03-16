use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use smol::Task;

pub struct JoinHandle<T> {
    pub task: Option<Task<T>>,
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.detach();
        }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.task.as_mut() {
            Some(task) => Future::poll(Pin::new(task), cx),
            None => unreachable!("JoinHandle polled after dropping"),
        }
    }
}
