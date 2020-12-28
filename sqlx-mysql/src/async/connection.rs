use futures_util::future::BoxFuture;
use sqlx_core::{Async, Connection, Result, Runtime};

use crate::{MySql, MySqlConnectOptions, MySqlConnection};

impl<Rt> Connection<Rt> for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    type Database = MySql;

    type Options = MySqlConnectOptions<Rt>;

    fn close(self) -> BoxFuture<'static, Result<()>>
    where
        Rt: Async,
    {
        unimplemented!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<()>>
    where
        Rt: Async,
    {
        unimplemented!()
    }
}
