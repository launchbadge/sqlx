use std::fmt::{self, Debug, Formatter};
use std::net::Shutdown;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use hashbrown::HashMap;

use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::mysql::protocol::text::{Ping, Quit};
use crate::mysql::row::MySqlColumn;
use crate::mysql::{MySql, MySqlConnectOptions, MySqlTypeInfo};

mod auth;
mod establish;
mod executor;
mod stream;
mod tls;

pub(crate) use stream::MySqlStream;

const COLLATE_UTF8MB4_UNICODE_CI: u8 = 224;

const MAX_PACKET_SIZE: u32 = 1024;

/// A connection to a MySQL database.
pub struct MySqlConnection {
    // underlying TCP stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    stream: MySqlStream,

    // cache by query string to the statement id
    cache_statement: HashMap<String, u32>,

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

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            self.stream.send_packet(Quit).await?;
            self.stream.shutdown(Shutdown::Both)?;

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
}

impl Connect for MySqlConnection {
    type Options = MySqlConnectOptions;

    #[inline]
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(MySqlConnection::establish(options))
    }
}
