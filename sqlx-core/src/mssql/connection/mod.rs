use crate::common::StatementCache;
use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::executor::Executor;
use crate::mssql::connection::stream::MssqlStream;
use crate::mssql::statement::MssqlStatementMetadata;
use crate::mssql::{Mssql, MssqlConnectOptions};
use crate::transaction::Transaction;
use futures_core::future::BoxFuture;
use futures_util::{future::ready, FutureExt, TryFutureExt};
use std::fmt::{self, Debug, Formatter};
use std::net::Shutdown;
use std::sync::Arc;

mod establish;
mod executor;
mod prepare;
mod stream;

pub struct MssqlConnection {
    pub(crate) stream: MssqlStream,
    pub(crate) cache_statement: StatementCache<Arc<MssqlStatementMetadata>>,
    log_settings: LogSettings,
}

impl Debug for MssqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MssqlConnection").finish()
    }
}

impl Connection for MssqlConnection {
    type Database = Mssql;

    type Options = MssqlConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        // NOTE: there does not seem to be a clean shutdown packet to send to MSSQL
        ready(self.stream.shutdown(Shutdown::Both).map_err(Into::into)).boxed()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // NOTE: we do not use `SELECT 1` as that *could* interact with any ongoing transactions
        self.execute("/* SQLx ping */").map_ok(|_| ()).boxed()
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.stream.wait_until_ready().boxed()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.wbuf.is_empty()
    }
}
