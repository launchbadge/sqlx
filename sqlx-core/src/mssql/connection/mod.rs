use std::fmt::{self, Debug, Formatter};
use std::net::Shutdown;

use futures_core::future::BoxFuture;
use futures_util::{future::ready, FutureExt, TryFutureExt};

use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::mssql::connection::stream::MssqlStream;
use crate::mssql::{Mssql, MssqlConnectOptions};

mod establish;
mod executor;
mod stream;

pub struct MssqlConnection {
    pub(crate) stream: MssqlStream,

    // number of Done* messages that we are currently expecting
    pub(crate) pending_done_count: usize,
}

impl Debug for MssqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MssqlConnection").finish()
    }
}

impl Connection for MssqlConnection {
    type Database = Mssql;

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
        self.wait_until_ready().boxed()
    }

    #[doc(hidden)]
    fn get_ref(&self) -> &MssqlConnection {
        self
    }

    #[doc(hidden)]
    fn get_mut(&mut self) -> &mut MssqlConnection {
        self
    }
}

impl Connect for MssqlConnection {
    type Options = MssqlConnectOptions;

    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(MssqlConnection::establish(options))
    }
}
