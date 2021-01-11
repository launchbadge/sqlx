use std::fmt::Debug;
use std::str::FromStr;

use crate::{Connection, Runtime};

/// Options which can be used to configure how a SQL connection is opened.
#[allow(clippy::module_name_repetitions)]
pub trait ConnectOptions<Rt>:
    'static + Sized + Send + Sync + Default + Debug + Clone + FromStr<Err = crate::Error>
where
    Rt: Runtime,
{
    type Connection: Connection<Rt> + ?Sized;

    /// Establish a new connection to the database.
    #[cfg(feature = "async")]
    fn connect(&self) -> futures_util::future::BoxFuture<'_, crate::Result<Self::Connection>>
    where
        Self::Connection: Sized,
        Rt: crate::Async;
}
