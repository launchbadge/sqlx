use super::Runtime;

pub trait Acquire<Rt>: crate::Acquire<Rt>
where
    Rt: Runtime,
{
    /// Get a connection from the pool, make a new connection, or wait for one to become
    /// available.
    ///
    /// Takes exclusive use of the connection until it is released.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`acquire()`][crate::Acquire::acquire].
    ///
    fn acquire(self) -> crate::Result<Self::Connection>
    where
        Self::Connection: Sized;

    fn begin(self) -> crate::Result<Self::Connection>
    where
        Self::Connection: Sized;

    fn try_begin(self) -> crate::Result<Option<Self::Connection>>
    where
        Self::Connection: Sized;
}
