use std::fmt::{self, Debug, Formatter};
use std::net::Shutdown;

use futures_core::future::BoxFuture;
use futures_util::{future::ready, FutureExt, TryFutureExt};

use crate::connection::{Connect, Connection};
use crate::error::{BoxDynError, Error};
use crate::executor::Executor;
use crate::mssql::connection::stream::MsSqlStream;
use crate::mssql::{MsSql, MsSqlConnectOptions};

mod establish;
mod executor;
mod stream;

pub struct MsSqlConnection {
    stream: MsSqlStream,

    // number of Done* messages that we are currently expecting
    pub(crate) pending_done_count: usize,
}

impl Debug for MsSqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MsSqlConnection").finish()
    }
}

impl Connection for MsSqlConnection {
    type Database = MsSql;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        // NOTE: there does not seem to be a clean shutdown packet to send to MSSQL
        ready(self.stream.shutdown(Shutdown::Both).map_err(Into::into)).boxed()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // NOTE: we do not use `SELECT 1` as that *could* interact with any ongoing transactions
        self.execute("/* SQLx ping */").map_ok(|_| ()).boxed()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    #[doc(hidden)]
    fn get_ref(&self) -> &MsSqlConnection {
        self
    }

    #[doc(hidden)]
    fn get_mut(&mut self) -> &mut MsSqlConnection {
        self
    }
}

impl Connect for MsSqlConnection {
    type Options = MsSqlConnectOptions;

    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(MsSqlConnection::establish(options))
    }
}
