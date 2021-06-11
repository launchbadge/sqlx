use futures_util::future::{poll_fn, BoxFuture};
use futures_util::pin_mut;
use std::error::Error as StdError;
use std::future::Future;
use std::task::Poll;

pub async fn assert_cancellation_safe<C, F1, F1R, F1E, F2, F2R, F2E>(
    mut context: C,
    task: F1,
    checkpoint: F2,
) -> anyhow::Result<()>
where
    F1: Fn(&mut C) -> BoxFuture<Result<F1R, F1E>>,
    F1E: 'static + StdError + Send + Sync,
    F2: Fn(&mut C) -> BoxFuture<Result<F2R, F2E>>,
    F2E: 'static + StdError + Send + Sync,
{
    for _ in 0..100 {
        for max_polls in 0.. {
            let mut num_polls = 0;

            {
                let fut = (task)(&mut context);
                pin_mut!(fut);

                let res = poll_fn(|ctx| {
                    let poll = match fut.as_mut().poll(ctx) {
                        Poll::Ready(it) => Poll::Ready(Some(it)),
                        Poll::Pending if num_polls == max_polls => Poll::Ready(None),
                        Poll::Pending => Poll::Pending,
                    };

                    num_polls += 1;

                    poll
                })
                .await;

                match res {
                    Some(Ok(_)) => break,
                    Some(Err(error)) => return Err(error.into()),
                    None => {}
                }
            }

            (checkpoint)(&mut context).await?;
        }
    }

    Ok(())
}
