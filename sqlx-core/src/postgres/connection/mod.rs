use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};
use std::ops::RangeFrom;
use std::sync::Arc;

use bytes::{Buf, Bytes};
use futures_core::future::BoxFuture;
use hashbrown::HashMap;

use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::io::{BufStream, Decode};
use crate::net::{MaybeTlsStream, Socket};
use crate::postgres::connection::describe::Statement;
use crate::postgres::connection::stream::PgStream;
use crate::postgres::message::{
    Message, MessageFormat, ReadyForQuery, Response, Terminate, TransactionStatus,
};
use crate::postgres::{PgConnectOptions, PgDatabaseError, Postgres};

pub(crate) mod describe;
mod establish;
mod executor;
mod stream;

/// A connection to a PostgreSQL database.
pub struct PgConnection {
    // underlying TCP or UDS stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    pub(crate) stream: PgStream,

    // process id of this backend
    // used to send cancel requests
    process_id: u32,

    // secret key of this backend
    // used to send cancel requests
    secret_key: u32,

    // sequence of statement IDs for use in preparing statements
    // in PostgreSQL, the statement is prepared to a user-supplied identifier
    next_statement_id: RangeFrom<u32>,

    // cache statement by query string to the id and columns
    cache_statement: HashMap<String, Arc<Statement>>,

    // cache user-defined types by id <-> name
    cache_type_name: HashMap<u32, UStr>,
    cache_type_id: HashMap<UStr, u32>,

    // number of ReadyForQuery messages that we are currently expecting
    pending_ready_for_query_count: usize,

    // current transaction status
    transaction_status: TransactionStatus,
}

impl PgConnection {
    // will return when the connection is ready for another query
    async fn wait_until_ready(&mut self) -> Result<(), Error> {
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
            self.stream.write(Terminate).await?;
            self.stream.flush().await?;
            self.stream.shutdown()?;

            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        todo!()
    }
}

impl Connect for PgConnection {
    type Options = PgConnectOptions;

    #[inline]
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(PgConnection::establish(options))
    }
}
