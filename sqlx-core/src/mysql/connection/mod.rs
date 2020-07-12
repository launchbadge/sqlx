use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use hashbrown::HashMap;

use crate::common::StatementCache;
use crate::connection::Connection;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::mysql::protocol::statement::StmtClose;
use crate::mysql::protocol::text::{Ping, Quit};
use crate::mysql::{MySql, MySqlColumn, MySqlConnectOptions};

mod auth;
mod establish;
mod executor;
mod stream;
mod tls;

use crate::transaction::Transaction;
pub(crate) use stream::{Busy, MySqlStream};

const COLLATE_UTF8MB4_UNICODE_CI: u8 = 224;

const MAX_PACKET_SIZE: u32 = 1024;

/// A connection to a MySQL database.
pub struct MySqlConnection {
    // underlying TCP stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    pub(crate) stream: MySqlStream,

    // transaction status
    pub(crate) transaction_depth: usize,

    // cache by query string to the statement id
    cache_statement: StatementCache<u32>,

    // working memory for the active row's column information
    // this allows us to re-use these allocations unless the user is persisting the
    // Row type past a stream iteration (clone-on-write)
    scratch_row_columns: Arc<Vec<MySqlColumn>>,
    scratch_row_column_names: Arc<HashMap<UStr, usize>>,
}

impl Debug for MySqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnection").finish()
    }
}

impl Connection for MySqlConnection {
    type Database = MySql;

    type Options = MySqlConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            self.stream.send_packet(Quit).await?;
            self.stream.shutdown()?;

            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.stream.wait_until_ready().await?;
            self.stream.send_packet(Ping).await?;
            self.stream.recv_ok().await?;

            Ok(())
        })
    }

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.stream.wait_until_ready().boxed()
    }

    fn cached_statements_size(&self) -> usize {
        self.cache_statement.len()
    }

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            while let Some(statement) = self.cache_statement.remove_lru() {
                self.stream.send_packet(StmtClose { statement }).await?;
            }

            Ok(())
        })
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.wbuf.is_empty()
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }
}
