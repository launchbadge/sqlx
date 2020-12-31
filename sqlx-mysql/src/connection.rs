use std::fmt::{self, Debug, Formatter};

use sqlx_core::{Connection, DefaultRuntime, Runtime};

use crate::{MySql, MySqlConnectOptions};

pub struct MySqlConnection<Rt = DefaultRuntime>
where
    Rt: Runtime,
{
    pub(crate) stream: Rt::TcpStream,
}

impl<Rt> Debug for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnection").finish()
    }
}

impl<Rt> Connection<Rt> for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    type Database = MySql;

    type Options = MySqlConnectOptions<Rt>;

    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        unimplemented!()
    }

    #[cfg(feature = "async")]
    fn ping(&mut self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::Async,
    {
        unimplemented!()
    }
}
