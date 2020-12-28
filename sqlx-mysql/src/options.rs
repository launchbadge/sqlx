use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;

use sqlx_core::{DefaultRuntime, Runtime};

pub struct MySqlConnectOptions<Rt = DefaultRuntime>
where
    Rt: Runtime,
{
    runtime: PhantomData<Rt>,
    pub(crate) host: String,
    pub(crate) port: u16,
}

impl<Rt> Default for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            runtime: PhantomData,
            port: 3306,
        }
    }
}

impl<Rt> Clone for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<Rt> Debug for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnectOptions").finish()
    }
}

impl<Rt> FromStr for MySqlConnectOptions<Rt>
where
    Rt: Runtime,
{
    type Err = sqlx_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            host: "localhost".to_owned(),
            runtime: PhantomData,
            port: 3306,
        })
    }
}
