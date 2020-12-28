use std::fmt::Debug;
use std::str::FromStr;

use futures_util::future::BoxFuture;

use crate::{Connection, Runtime};

/// Options which can be used to configure how a SQL connection is opened.
pub trait ConnectOptions<Rt>:
    'static + Send + Sync + Default + Debug + Clone + FromStr<Err = crate::Error>
where
    Rt: Runtime,
{
    type Connection: Connection<Rt> + ?Sized;

    /// Establish a connection to the database.
    fn connect(&self) -> BoxFuture<'_, crate::Result<Self::Connection>>
    where
        Self::Connection: Sized;
}
