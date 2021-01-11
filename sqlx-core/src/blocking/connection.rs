use super::{Close, Connect, ConnectOptions, Runtime};

/// A unique connection (session) with a specific database.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`Connection`][crate::Connection].
///
pub trait Connection<Rt>: crate::Connection<Rt> + Close<Rt> + Connect<Rt>
where
    Rt: Runtime,
    Self::Options: ConnectOptions<Rt>,
{
    /// Checks if a connection to the database is still valid.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`ping()`][crate::Connection::ping].
    ///
    fn ping(&mut self) -> crate::Result<()>;
}
