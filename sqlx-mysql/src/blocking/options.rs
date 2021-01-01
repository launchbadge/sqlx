use sqlx_core::blocking::{ConnectOptions, Connection, Runtime};
use sqlx_core::Result;

use crate::{MySqlConnectOptions, MySqlConnection};

impl<Rt> ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
    Self::Connection: sqlx_core::Connection<Rt, Options = Self> + Connection<Rt>,
{
    fn connect(&self) -> Result<MySqlConnection<Rt>> {
        // let stream = <Rt as Runtime>::connect_tcp(self.get_host(), self.get_port())?;
        //
        // Ok(MySqlConnection { stream })
        todo!()
    }
}
