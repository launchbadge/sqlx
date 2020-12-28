use sqlx_core::blocking::{ConnectOptions, Runtime};
use sqlx_core::Result;

use crate::{MySqlConnectOptions, MySqlConnection};

impl<Rt> ConnectOptions<Rt> for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    type Connection = MySqlConnection<Rt>;

    fn connect(&self) -> Result<MySqlConnection<Rt>> {
        let stream = <Rt as Runtime>::connect_tcp(&self.host, self.port)?;

        Ok(MySqlConnection { stream })
    }
}
