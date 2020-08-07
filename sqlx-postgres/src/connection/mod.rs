use crate::codec::backend::{MessageFormat, ReadyForQuery, TransactionStatus};
use crate::statement::StatementMetadata;
use crate::{PgConnectOptions, Postgres};
use futures_core::future::BoxFuture;
use sqlx_core::cache::StringCache;
use sqlx_core::connection::Connection;
use sqlx_core::error::Error;
use sqlx_core::execute::Execute;
use sqlx_core::io::BufStream;
use sqlx_rt::TcpStream;
use std::sync::Arc;

mod connect;
mod executor;
mod io;

/// A connection to a PostgreSQL database.
pub struct PgConnection {
    // underlying TCP or UDS stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    stream: BufStream<TcpStream>,

    // process id of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    process_id: u32,

    // secret key of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    secret_key: u32,

    // status of the connection
    // are we in a transaction?
    transaction_status: TransactionStatus,

    // sequence of statement IDs for use in preparing statements
    // in PostgreSQL, the statement is prepared to a user-supplied identifier
    next_statement_id: u32,

    // cache of SQL query -> statement ID
    cache_statement: StringCache<u32>,

    // cache of SQL query -> statement metadata
    cache_metadata: StringCache<Arc<StatementMetadata>>,

    // number of [ReadyForQuery] messages until the connection buffer is fully drained
    pending_ready_for_query: usize,
}

impl PgConnection {
    pub(crate) fn new(stream: BufStream<TcpStream>, options: &PgConnectOptions) -> Self {
        Self {
            stream,
            process_id: 0,
            secret_key: 0,
            transaction_status: TransactionStatus::Idle,
            pending_ready_for_query: 0,
            next_statement_id: 1,
            cache_statement: StringCache::new(options.statement_cache_capacity),
            cache_metadata: StringCache::new(options.metadata_cache_capacity),
        }
    }

    pub(crate) fn handle_ready_for_query(&mut self, ready: ReadyForQuery) {
        self.pending_ready_for_query -= 1;
        self.transaction_status = ready.transaction_status;
    }

    // drain connection buffer until we consume any pending [ReadyForQuery] messages
    // we get into this state when a fetch stream is dropped before being read to completion
    pub(crate) async fn drain(&mut self) -> Result<(), Error> {
        while self.pending_ready_for_query > 0 {
            // TODO: database errors should *not* be returned as a database error means that
            //       the database returned an error in a cancelled or dropped query
            let message = self.recv_exact(MessageFormat::ReadyForQuery).await?;
            self.handle_ready_for_query(message.decode()?);
        }

        Ok(())
    }
}

impl Connection for PgConnection {
    type Database = Postgres;

    type Options = PgConnectOptions;

    fn execute<'x, 'c: 'x, 'q: 'x, E: 'x + Execute<'q, Postgres>>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'x, Result<u64, Error>> {
        Box::pin(self.execute(query))
    }

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        unimplemented!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.drain())
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        unimplemented!()
    }
}
