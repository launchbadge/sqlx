use super::{Connect, Runtime};

/// Options which can be used to configure how a SQL connection is opened.
///
/// For detailed information, refer to the async version of
/// this: [`ConnectOptions`][crate::ConnectOptions].
///
#[allow(clippy::module_name_repetitions)]
pub trait ConnectOptions: crate::ConnectOptions {
    /// Establish a connection to the database.
    ///
    /// For detailed information, refer to the async version of
    /// this: [`connect()`][crate::ConnectOptions::connect].
    ///
    fn connect<C, Rt>(&self) -> crate::Result<C>
    where
        C: Connect<Rt, Options = Self> + Sized,
        Rt: Runtime,
    {
        <C as Connect<Rt>>::connect_with(self)
    }
}
