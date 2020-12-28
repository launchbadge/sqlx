use std::fmt::{self, Debug, Formatter};

use sqlx_core::{DefaultRuntime, Runtime};

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
