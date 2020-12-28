use super::Runtime;

/// Options which can be used to configure how a SQL connection is opened.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`ConnectOptions`][crate::ConnectOptions].
///
pub trait ConnectOptions<Rt>: crate::ConnectOptions<Rt>
where
    Rt: Runtime,
{
    /// Establish a connection to the database.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`connect()`][crate::ConnectOptions::connect].
    ///
    fn connect(&self) -> crate::Result<Self::Connection>
    where
        Self::Connection: Sized;
}
