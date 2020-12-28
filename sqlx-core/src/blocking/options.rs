use super::{Connection, Runtime};
use crate::DefaultRuntime;

/// Options which can be used to configure how a SQL connection is opened.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`ConnectOptions`][crate::ConnectOptions].
///
pub trait ConnectOptions<Rt = DefaultRuntime>: crate::ConnectOptions<Rt>
where
    Rt: Runtime,
{
    type Connection: Connection<Rt> + ?Sized;

    /// Establish a connection to the database.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`connect()`][crate::ConnectOptions::connect].
    ///
    fn connect(&self) -> crate::Result<<Self as ConnectOptions<Rt>>::Connection>
    where
        <Self as ConnectOptions<Rt>>::Connection: Sized;
}
