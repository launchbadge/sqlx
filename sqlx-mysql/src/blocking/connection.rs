use sqlx_core::blocking::{Connection, Runtime};
use sqlx_core::Result;

use crate::{MySqlConnectOptions, MySqlConnection};

impl<Rt> Connection<Rt> for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    type Options = MySqlConnectOptions<Rt>;

    fn close(self) -> Result<()> {
        unimplemented!()
    }

    fn ping(&mut self) -> Result<()> {
        unimplemented!()
    }
}
