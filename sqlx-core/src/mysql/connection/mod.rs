use std::fmt::{self, Debug, Formatter};
use std::net::Shutdown;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use hashbrown::HashMap;

use crate::caching_connection::CachingConnection;
use crate::common::StatementCache;
use crate::connection::{Connect, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::ext::ustr::UStr;
use crate::mysql::protocol::statement::StmtClose;
use crate::mysql::protocol::text::{Ping, Quit};
use crate::mysql::row::MySqlColumn;
use crate::mysql::{MySql, MySqlConnectOptions};

mod auth;
mod establish;
mod executor;
mod stream;
mod tls;

pub(crate) use stream::{Busy, MySqlStream};

const COLLATE_UTF8MB4_UNICODE_CI: u8 = 224;

const MAX_PACKET_SIZE: u32 = 1024;

/// A connection to a MySQL database.
pub struct MySqlConnection {
    // underlying TCP stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    pub(crate) stream: MySqlStream,

    // cache by query string to the statement id
    cache_statement: StatementCache<u32>,

    // working memory for the active row's column information
    // this allows us to re-use these allocations unless the user is persisting the
    // Row type past a stream iteration (clone-on-write)
    scratch_row_columns: Arc<Vec<MySqlColumn>>,
    scratch_row_column_names: Arc<HashMap<UStr, usize>>,
}

impl CachingConnection for MySqlConnection {
    fn cached_statements_count(&self) -> usize {
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

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.stream.wait_until_ready().boxed()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.stream.wbuf.is_empty()
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

impl Connect for MySqlConnection {
    type Options = MySqlConnectOptions;

    #[inline]
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(async move {
            let mut conn = MySqlConnection::establish(options).await?;

            // After the connection is established, we initialize by configuring a few
            // connection parameters

            // https://mariadb.com/kb/en/sql-mode/

            // PIPES_AS_CONCAT - Allows using the pipe character (ASCII 124) as string concatenation operator.
            //                   This means that "A" || "B" can be used in place of CONCAT("A", "B").

            // NO_ENGINE_SUBSTITUTION - If not set, if the available storage engine specified by a CREATE TABLE is
            //                          not available, a warning is given and the default storage
            //                          engine is used instead.

            // NO_ZERO_DATE - Don't allow '0000-00-00'. This is invalid in Rust.

            // NO_ZERO_IN_DATE - Don't allow 'YYYY-00-00'. This is invalid in Rust.

            // --

            // Setting the time zone allows us to assume that the output
            // from a TIMESTAMP field is UTC

            // --

            // https://mathiasbynens.be/notes/mysql-utf8mb4

            conn.execute(r#"
            SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT,NO_ENGINE_SUBSTITUTION,NO_ZERO_DATE,NO_ZERO_IN_DATE'));
            SET time_zone = '+00:00';
            SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci;
                    "#).await?;

            Ok(conn)
        })
    }
}
