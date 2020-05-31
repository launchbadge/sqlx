use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::{FutureExt, TryFutureExt};
use hashbrown::HashMap;

use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::ext::ustr::UStr;
use crate::io::Decode;
use crate::postgres::connection::stream::PgStream;
use crate::postgres::message::{
    Message, MessageFormat, ReadyForQuery, Terminate, TransactionStatus,
};
use crate::postgres::row::PgColumn;
use crate::postgres::{PgConnectOptions, PgTypeInfo, Postgres};

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
    next_statement_id: u32,

    // cache statement by query string to the id and columns
    cache_statement: HashMap<String, u32>,

    // cache user-defined types by id <-> info
    cache_type_info: HashMap<u32, PgTypeInfo>,
    cache_type_oid: HashMap<UStr, u32>,

    // number of ReadyForQuery messages that we are currently expecting
    pub(crate) pending_ready_for_query_count: usize,

    // current transaction status
    transaction_status: TransactionStatus,

    // working memory for the active row's column information
    scratch_row_columns: Arc<Vec<PgColumn>>,
    scratch_row_column_names: Arc<HashMap<UStr, usize>>,
}

impl PgConnection {
    // will return when the connection is ready for another query
    async fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.stream.wbuf.is_empty() {
            self.stream.flush().await?;
        }

        while self.pending_ready_for_query_count > 0 {
            loop {
                let message = self.stream.recv().await?;

                match message.format {
                    MessageFormat::ReadyForQuery => {
                        self.handle_ready_for_query(message)?;
                        break;
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn handle_ready_for_query(&mut self, message: Message) -> Result<(), Error> {
        self.pending_ready_for_query_count -= 1;
        self.transaction_status = ReadyForQuery::decode(message.contents)?.transaction_status;

        Ok(())
    }
}

impl Debug for PgConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnection").finish()
    }
}

impl Connection for PgConnection {
    type Database = Postgres;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        // The normal, graceful termination procedure is that the frontend sends a Terminate
        // message and immediately closes the connection.

        // On receipt of this message, the backend closes the
        // connection and terminates.

        Box::pin(async move {
            self.stream.send(Terminate).await?;
            self.stream.shutdown()?;

            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.execute("SELECT 1").map_ok(|_| ()).boxed()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.wait_until_ready().boxed()
    }

    #[doc(hidden)]
    fn get_ref(&self) -> &Self {
        self
    }

    #[doc(hidden)]
    fn get_mut(&mut self) -> &mut Self {
        self
    }
}

impl Connect for PgConnection {
    type Options = PgConnectOptions;

    #[inline]
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(PgConnection::establish(options))
    }
}
