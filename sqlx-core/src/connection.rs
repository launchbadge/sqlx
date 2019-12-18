use futures_core::future::BoxFuture;
use crate::Executor;

pub trait Connection: Executor + Sized {
    /// Gracefully close the connection.
    fn close(self) -> BoxFuture<'static, crate::Result<()>>;
}
