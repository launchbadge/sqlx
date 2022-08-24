use crate::connection::LogSettings;
use crate::error::Error;
use crate::executor::Execute;
use crate::logger::QueryLogger;
use crate::postgres::statement::PgStatementMetadata;
use crate::postgres::{PgArguments, PgPool, PgQueryResult, Postgres};
use either::Either;
use futures_core::Stream;
use futures_util::{stream, StreamExt, TryStreamExt};
use smallvec::SmallVec;
use std::sync::Arc;

// tuple that contains the data required to run a query
//
// (sql, arguments, persistent, metadata_options)
type QueryContext<'q> = (
    &'q str,
    Option<PgArguments>,
    bool,
    Option<Arc<PgStatementMetadata>>,
);

/// Pipeline of independent queries.
///
/// Query pipeline allows to issue multiple independent queries via extended
/// query protocol in a single batch, write Sync command and wait for the result sets of all queries.
///
/// If there is no explicit transaction than queries will run in an implicit
/// transaction with shortest possible duration.
///
/// It's assumed that the queries produce small enough result sets that together
/// fit in client's memory. Pipeline doesn't use server side cursors.
///
/// Simple queries are not supported due to focus on efficient execution of the
/// same queries with different parameters.
/// Though technically the implicit transaction may commit by a single simple
/// Query instead of the final Sync.
///
/// CockroachDB specifics: This transaction could be automatically retried by
/// the database gateway node during contention with other transactions as long as it can
/// buffer all result sets (see
/// https://www.cockroachlabs.com/docs/stable/transactions.html#automatic-retries).
pub struct PgExtendedQueryPipeline<'q, const N: usize> {
    queries: SmallVec<[QueryContext<'q>; N]>,
}

impl<'q, const N: usize> PgExtendedQueryPipeline<'q, N> {
    pub fn push(&mut self, mut query: impl Execute<'q, Postgres>) {
        self.queries.push((
            query.sql(),
            query.take_arguments(),
            query.persistent(),
            query.statement().map(|s| Arc::clone(&s.metadata)),
        ))
    }

    pub async fn execute(
        self: PgExtendedQueryPipeline<'q, N>,
        pool: &PgPool,
    ) -> Result<SmallVec<[PgQueryResult; N]>, Error> {
        let mut conn = pool.acquire().await?;
        let pgresults = conn
            .fetch_pipeline(self)
            .await?
            .filter_map(|pgresult_or_row_result| async move {
                match pgresult_or_row_result {
                    Ok(Either::Left(pgresult)) => Some(Ok(pgresult)),
                    // filter data rows
                    Ok(Either::Right(_)) => None,
                    Err(e) => Some(Err(e)),
                }
            })
            .try_collect()
            .await?;
        Ok(pgresults)
    }

    pub(crate) fn len(&self) -> usize {
        self.queries.len()
    }

    pub(crate) fn query_loggers_stack(
        &self,
        log_settings: &LogSettings,
    ) -> SmallVec<[QueryLogger<'q>; N]> {
        self.queries
            .iter()
            .rev()
            .map(|(q_ref, _, _, _)| QueryLogger::new(*q_ref, log_settings.clone()))
            .collect()
    }

    pub(crate) fn into_querycontext_stream(self) -> impl Stream<Item = QueryContext<'q>> {
        stream::iter(self.queries)
    }
}

impl<'q, E, const N: usize> From<E> for PgExtendedQueryPipeline<'q, N>
where
    E: Execute<'q, Postgres>,
{
    /// Query pipeline has at least one query.
    fn from(query: E) -> Self {
        let mut pipeline = Self {
            queries: SmallVec::new(),
        };
        pipeline.push(query);
        pipeline
    }
}
