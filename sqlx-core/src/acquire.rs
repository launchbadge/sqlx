#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

use crate::{Connection, Runtime};

#[allow(clippy::type_complexity)]
pub trait Acquire<Rt: Runtime> {
    type Connection: Connection<Rt>;

    /// Get a connection from the pool, make a new connection, or wait for one to become
    /// available.
    ///
    /// Takes exclusive use of the connection until it is released.
    ///
    #[cfg(feature = "async")]
    fn acquire(self) -> BoxFuture<'static, crate::Result<Self::Connection>>
    where
        Self::Connection: Sized;

    /// Get a connection from the pool, if available.
    ///
    /// Returns `None` immediately if there are no connections available.
    ///
    fn try_acquire(self) -> Option<Self::Connection>
    where
        Self::Connection: Sized;

    #[cfg(feature = "async")]
    fn begin(self) -> BoxFuture<'static, crate::Result<Self::Connection>>
    where
        Self::Connection: Sized;

    #[cfg(feature = "async")]
    fn try_begin(self) -> BoxFuture<'static, crate::Result<Option<Self::Connection>>>
    where
        Self::Connection: Sized;
}

// TODO: impl Acquire for &Pool { ... }
// TODO: impl<C: Connection> Acquire for &mut C { ... }
// TODO: impl<A: Acquire> Acquire for &mut &A { ... }
