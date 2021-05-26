use crate::{Executor, Runtime, Database};
#[cfg(feature = "async")]
use futures_util::future::{BoxFuture};
#[cfg(feature = "async")]
use crate::Async;

pub trait Transaction<'t, Db: Database, Rt: Runtime>: Executor<Rt> {
    /// Commit this transaction to the database
    #[cfg(feature = "async")]
    fn commit(self) -> BoxFuture<'t, crate::Result<()>> where Rt: Async;
    /// Abort this transaction without committing, and roll back to the last save point (if any) or the start of the transaction (if no save points are present)
    #[cfg(feature = "async")]
    fn abort(self) -> BoxFuture<'t, crate::Result<()>> where Rt: Async;
    /// Roll this transaction back to the last save point (if any) or the start of the transaction (if no save points are present). This function can panic if the database backend does not support this operation.
    #[cfg(feature = "async")]
    fn rollback(&'t self) -> BoxFuture<'t, crate::Result<()>> where Rt: Async;
    /// Create a save point, if supported by the current database backend, panic otherwise
    #[cfg(feature = "async")]
    fn save(&'t self) -> BoxFuture<'t, crate::Result<()>> where Rt: Async;
    /// Check if save points are supported by the current database backend
    fn save_point_supported(&self) -> bool;
}

pub trait TransactionSource<'t, Db: Database, Rt: Runtime, Arg> {
    type Transaction: Transaction<'t, Db, Rt>;

    #[inline]
    #[cfg(feature = "async")]
    fn begin(&'t mut self) -> BoxFuture<'t, crate::Result<Self::Transaction>> where Arg: Default{
        self.begin_arg(Arg::default())
    }

    #[cfg(feature = "async")]
    fn begin_arg(&'t mut self, arg: Arg) -> BoxFuture<'t, crate::Result<Self::Transaction>>;

}