#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

use crate::{Database, DefaultRuntime, Runtime};

pub trait Acquire<Rt: Runtime = DefaultRuntime> {
    type Database: Database<Rt>;

    /// Get a connection from the pool, make a new connection, or wait for one to become
    /// available.
    ///
    /// Takes exclusive use of the connection until it is released.
    ///
    #[cfg(feature = "async")]
    fn acquire(
        self,
    ) -> BoxFuture<'static, crate::Result<<Self::Database as Database<Rt>>::Connection>>
    where
        <Self::Database as Database<Rt>>::Connection: Sized;

    /// Get a connection from the pool, if available.
    ///
    /// Returns `None` immediately if there are no connections available.
    ///  
    fn try_acquire(self) -> Option<<Self::Database as Database<Rt>>::Connection>
    where
        <Self::Database as Database<Rt>>::Connection: Sized;

    #[cfg(feature = "async")]
    fn begin(
        self,
    ) -> BoxFuture<'static, crate::Result<<Self::Database as Database<Rt>>::Connection>>
    where
        <Self::Database as Database<Rt>>::Connection: Sized;

    #[cfg(feature = "async")]
    fn try_begin(
        self,
    ) -> BoxFuture<'static, crate::Result<Option<<Self::Database as Database<Rt>>::Connection>>>
    where
        <Self::Database as Database<Rt>>::Connection: Sized;
}

// TODO: impl Acquire for &Pool { ... }
// TODO: impl<C: Connection> Acquire for &mut C { ... }
// TODO: impl<A: Acquire> Acquire for &mut &A { ... }
