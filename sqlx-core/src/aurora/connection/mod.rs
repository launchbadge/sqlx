use crate::aurora::options::{AuroraConnectOptions, AuroraDbType};
use crate::aurora::statement::AuroraStatementMetadata;
use crate::aurora::Aurora;
use crate::common::StatementCache;
use crate::connection::Connection;
use crate::connection::LogSettings;
use crate::error::Error;
use crate::transaction::Transaction;

use futures_core::future::BoxFuture;
use futures_util::future;
use rusoto_rds_data::RdsDataClient;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

mod executor;

/// A connection to an Aurora database.
pub struct AuroraConnection {
    pub(crate) db_type: AuroraDbType,
    pub(crate) resource_arn: String,
    pub(crate) secret_arn: String,
    pub(crate) database: Option<String>,
    pub(crate) schema: Option<String>,

    pub(crate) client: RdsDataClient,

    // Transaction identifiers
    pub(crate) transaction_ids: Vec<String>,

    // cache statement by query string to the id and columns
    cache_statement: StatementCache<(u32, Arc<AuroraStatementMetadata>)>,

    log_settings: LogSettings,
}

impl AuroraConnection {
    pub(crate) fn new(options: &AuroraConnectOptions) -> Result<Self, Error> {
        let db_type = options
            .db_type
            .ok_or_else(|| Error::Configuration("db type not specified".into()))?;

        let region = options.region.parse().map_err(Error::config)?;

        let resource_arn = options
            .resource_arn
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::Configuration("Resource ARN not specified".into()))?;
        let secret_arn = options
            .secret_arn
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::Configuration("Secret ARN not specified".into()))?;

        let client = RdsDataClient::new(region);

        Ok(Self {
            db_type,
            resource_arn,
            secret_arn,
            database: options.database.clone(),
            schema: options.schema.clone(),
            client,
            transaction_ids: vec![],
            cache_statement: StatementCache::new(options.statement_cache_capacity),
            log_settings: options.log_settings.clone(),
        })
    }
}

impl Debug for AuroraConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuroraConnection").finish()
    }
}

impl Connection for AuroraConnection {
    type Database = Aurora;

    type Options = AuroraConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        // nothing explicit to do; connection will close in drop
        Box::pin(future::ok(()))
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For Aurora connections, PING does effectively nothing
        Box::pin(future::ok(()))
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
            self.cache_statement.clear();
            Ok(())
        })
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // For SQLite, FLUSH does effectively nothing
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        false
    }
}
