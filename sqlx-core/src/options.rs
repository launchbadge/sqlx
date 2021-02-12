use std::fmt::Debug;
use std::str::FromStr;

use crate::Connect;

/// Options which can be used to configure how a SQL connection is opened.
#[allow(clippy::module_name_repetitions)]
pub trait ConnectOptions:
    'static + Sized + Send + Sync + Default + Debug + Clone + FromStr<Err = crate::Error>
{
    /// Establish a new connection to the database.
    #[cfg(feature = "async")]
    fn connect<C, Rt>(&self) -> futures_util::future::BoxFuture<'_, crate::Result<C>>
    where
        C: Connect<Rt, Options = Self> + Sized,
        Rt: crate::Async,
    {
        C::connect_with(self)
    }
}
