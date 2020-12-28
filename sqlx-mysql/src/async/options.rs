use futures_util::{future::BoxFuture, FutureExt};
use sqlx_core::{Async, ConnectOptions, Result, Runtime};

use crate::{MySqlConnectOptions, MySqlConnection};

impl<Rt> ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    type Connection = MySqlConnection<Rt>;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection>>
    where
        Self::Connection: Sized,
        Rt: Async,
    {
        FutureExt::boxed(async move {
            let stream = Rt::connect_tcp(&self.host, self.port).await?;

            Ok(MySqlConnection { stream })
        })
    }
}
