use std::fmt::Debug;
use std::str::FromStr;

use super::{Connection, Runtime};

/// Options which can be used to configure how a SQL connection is opened.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`ConnectOptions`][crate::ConnectOptions].
///
pub trait ConnectOptions<Rt>:
    'static + Send + Sync + Default + Debug + Clone + FromStr<Err = crate::Error>
where
    Rt: Runtime,
{
    type Connection: Connection<Rt> + ?Sized;

    /// Establish a connection to the database.
    ///
    /// For detailed information, refer to the asynchronous version of
    /// this: [`connect()`][crate::ConnectOptions::connect].
    ///
    fn connect(&self) -> crate::Result<Self::Connection>
    where
        Self::Connection: Sized;
}
