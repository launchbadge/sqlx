use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use crate::HashMap;
use futures_core::future::BoxFuture;
use futures_util::FutureExt;

use crate::common::StatementCache;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::io::Decode;
use crate::message::{
    Close, Message, MessageFormat, Query, ReadyForQuery, Terminate, TransactionStatus,
};
use crate::statement::PgStatementMetadata;
use crate::transaction::Transaction;
use crate::types::Oid;
use crate::{PgConnectOptions, PgTypeInfo, Postgres};

pub(crate) use sqlx_core::connection::*;

pub use self::stream::PgStream;

pub(crate) mod describe;
mod establish;
mod executor;
mod sasl;
mod stream;
mod tls;

/// A connection to a PostgreSQL database.
pub struct PgConnection {
    // underlying TCP or UDS stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    pub(crate) stream: PgStream,

    // process id of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    process_id: u32,

    // secret key of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    secret_key: u32,

    // sequence of statement IDs for use in preparing statements
    // in PostgreSQL, the statement is prepared to a user-supplied identifier
    next_statement_id: Oid,

    // cache statement by query string to the id and columns
    cache_statement: StatementCache<(Oid, Arc<PgStatementMetadata>)>,

    // cache user-defined types by id <-> info
    cache_type_info: HashMap<Oid, PgTypeInfo>,
    cache_type_oid: HashMap<UStr, Oid>,
    cache_elem_type_to_array: HashMap<Oid, Oid>,

    // number of ReadyForQuery messages that we are currently expecting
    pub(crate) pending_ready_for_query_count: usize,

    // current transaction status
    transaction_status: TransactionStatus,
    pub(crate) transaction_depth: usize,

    log_settings: LogSettings,
}

impl PgConnection {
    /// the version number of the server in `libpq` format
    pub fn server_version_num(&self) -> Option<u32> {
        self.stream.server_version_num
    }

    // will return when the connection is ready for another query
    pub(crate) async fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.stream.write_buffer_mut().is_empty() {
            self.stream.flush().await?;
        }

        while self.pending_ready_for_query_count > 0 {
            let message = self.stream.recv().await?;

            if let MessageFormat::ReadyForQuery = message.format {
                self.handle_ready_for_query(message)?;
            }
        }

        Ok(())
    }

    async fn recv_ready_for_query(&mut self) -> Result<(), Error> {
        let r: ReadyForQuery = self
            .stream
            .recv_expect(MessageFormat::ReadyForQuery)
            .await?;

        self.pending_ready_for_query_count -= 1;
        self.transaction_status = r.transaction_status;

        Ok(())
    }

    fn handle_ready_for_query(&mut self, message: Message) -> Result<(), Error> {
        self.pending_ready_for_query_count -= 1;
        self.transaction_status = ReadyForQuery::decode(message.contents)?.transaction_status;

        Ok(())
    }

    /// Queue a simple query (not prepared) to execute the next time this connection is used.
    ///
    /// Used for rolling back transactions and releasing advisory locks.
    pub(crate) fn queue_simple_query(&mut self, query: &str) {
        self.pending_ready_for_query_count += 1;
        self.stream.write(Query(query));
    }
}

impl Debug for PgConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnection").finish()
    }
}

impl Connection for PgConnection {
    type Database = Postgres;

    type Options = PgConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        // The normal, graceful termination procedure is that the frontend sends a Terminate
        // message and immediately closes the connection.

        // On receipt of this message, the backend closes the
        // connection and terminates.

        Box::pin(async move {
            self.stream.send(Terminate).await?;
            self.stream.shutdown().await?;

            Ok(())
        })
    }

    fn close_hard(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            self.stream.shutdown().await?;

            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // Users were complaining about this showing up in query statistics on the server.
        // By sending a comment we avoid an error if the connection was in the middle of a rowset
        // self.execute("/* SQLx ping */").map_ok(|_| ()).boxed()

        Box::pin(async move {
            // The simplest call-and-response that's possible.
            self.write_sync();
            self.wait_until_ready().await
        })
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    fn cached_statements_size(&self) -> usize {
        self.cache_statement.len()
    }

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.cache_type_oid.clear();

            let mut cleared = 0_usize;

            self.wait_until_ready().await?;

            while let Some((id, _)) = self.cache_statement.remove_lru() {
                self.stream.write(Close::Statement(id));
                cleared += 1;
            }

            if cleared > 0 {
                self.write_sync();
                self.stream.flush().await?;

                self.wait_for_close_complete(cleared).await?;
                self.recv_ready_for_query().await?;
            }

            Ok(())
        })
    }

    fn shrink_buffers(&mut self) {
        self.stream.shrink_buffers();
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.wait_until_ready().boxed()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.write_buffer().is_empty()
    }
}

// Implement `AsMut<Self>` so that `PgConnection` can be wrapped in
// a `PgAdvisoryLockGuard`.
//
// See: https://github.com/launchbadge/sqlx/issues/2520
impl AsMut<PgConnection> for PgConnection {
    fn as_mut(&mut self) -> &mut PgConnection {
        self
    }
}
