use sqlx_core::blocking::{Connection, Runtime};
use sqlx_core::Result;

use crate::MySqlConnection;

impl<Rt> Connection<Rt> for MySqlConnection<Rt>
where
    Rt: Runtime,
{
    fn close(self) -> Result<()> {
        unimplemented!()
    }

    fn ping(&mut self) -> Result<()> {
        unimplemented!()
    }
}
