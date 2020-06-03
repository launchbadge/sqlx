use std::fmt::{self, Debug, Formatter};

use futures_core::future::BoxFuture;

use crate::connection::{Connect, Connection};
use crate::error::{BoxDynError, Error};
use crate::mssql::connection::stream::MsSqlStream;
use crate::mssql::{MsSql, MsSqlConnectOptions};

mod establish;
mod executor;
mod stream;

pub struct MsSqlConnection {
    stream: MsSqlStream,
}

impl Debug for MsSqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MsSqlConnection").finish()
    }
}

impl Connection for MsSqlConnection {
    type Database = MsSql;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        unimplemented!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    fn get_ref(&self) -> &MsSqlConnection {
        unimplemented!()
    }

    fn get_mut(&mut self) -> &mut MsSqlConnection {
        unimplemented!()
    }
}

impl Connect for MsSqlConnection {
    type Options = MsSqlConnectOptions;

    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(MsSqlConnection::establish(options))
    }
}
