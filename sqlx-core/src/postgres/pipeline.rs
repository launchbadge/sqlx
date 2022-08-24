use crate::error::Error;
use crate::executor::Execute;
use crate::logger::QueryLogger;
use crate::postgres::message::{self, Bind, CommandComplete, DataRow, MessageFormat};
use crate::postgres::statement::PgStatementMetadata;
use crate::postgres::types::Oid;
use crate::postgres::{
    PgArguments, PgConnection, PgPool, PgQueryResult, PgRow, PgValueFormat, Postgres,
};
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
/// Pipeline queries run on the same physical database connection.
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
///
/// [PgExtendedQueryPipeline] has `N` type parameter that defines the expected
/// maximum number of pipeline queries.  This number is used for stack
/// allocations.
///
#[cfg_attr(
    feature = "_rt-tokio",
    doc = r##"
# Example usage

```no_run
use sqlx::postgres::PgExtendedQueryPipeline;
use sqlx::PgPool;
use uuid::{uuid, Uuid};

#[tokio::main]
async fn main() -> sqlx::Result<()> {
    let pool = PgPool::connect("postgres://user@postgres/db").await?;

    let user_id = uuid!("6592b7c0-b531-4613-ace5-94246b7ce0c3");
    let post_id = uuid!("252c1d98-a9b0-4f18-8298-e59058bdfe16");
    let comment_id = uuid!("fbbbb7dc-dc6f-4649-b663-8d3636035164");

    let user_insert_query = sqlx::query(
        "
        INSERT INTO \"user\" (user_id, username)
        VALUES
        ($1, $2)
    ",
    )
    .bind(user_id)
    .bind("alice");

    const EXPECTED_QUERIES_IN_PIPELINE: usize = 3;
    let mut pipeline =
        PgExtendedQueryPipeline::<EXPECTED_QUERIES_IN_PIPELINE>::from(user_insert_query);

    // query without parameters
    let post_insert_query = sqlx::query(
        "
        INSERT INTO post (post_id, user_id, content)
        VALUES
        ('252c1d98-a9b0-4f18-8298-e59058bdfe16', '6592b7c0-b531-4613-ace5-94246b7ce0c3', 'test post')
    ",
    );

    pipeline.push(post_insert_query);

    let comment_insert_query = sqlx::query(
        "
        INSERT INTO comment (comment_id, post_id, user_id, content)
        VALUES
        ($1, $2, $3, $4)
    ",
    )
    .bind(comment_id)
    .bind(post_id)
    .bind(user_id)
    .bind("test comment");

    pipeline.push(comment_insert_query);
    let _ = pipeline.execute(&pool).await?;
    Ok(())
}
```
"##
)]
/// # Operations
/// There are two public operations available on pipelines:
///
/// * Execute
/// * Fetch
///
/// `Execute` filters any returned data rows and returns only a vector of
/// PgQueryResult structures.
/// `Execute` is available as [PgExtendedQueryPipeline::execute] method and implemented as `execute_pipeline` method
/// for the following:
///
///  * [`&PgPool`](super::PgPool)
///  * [`&mut PgConnection`](super::connection::PgConnection)
///
/// `Transaction` instance proxies `execute_pipeline` method to the underlying `PgConnection`.

/// `Fetch` returns a stream of either [PgQueryResult] or [PgRow] structures.
/// PgQueryResult structures.
/// `Fetch` is implemented as `fetch_pipeline` method for [`&mut PgConnection`](super::connection::PgConnection)
///
/// `Transaction` instance proxies `fetch_pipeline` method to the underlying [PgConnection].
///

// public interface section; private section is below
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
        pool.execute_pipeline(self).await
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

impl PgPool {
    pub async fn execute_pipeline<'q, const N: usize>(
        &self,
        pipeline: PgExtendedQueryPipeline<'q, N>,
    ) -> Result<SmallVec<[PgQueryResult; N]>, Error> {
        let mut conn = self.acquire().await?;
        conn.execute_pipeline(pipeline).await
    }
}

impl PgConnection {
    pub async fn execute_pipeline<'q, const N: usize>(
        &mut self,
        pipeline: PgExtendedQueryPipeline<'q, N>,
    ) -> Result<SmallVec<[PgQueryResult; N]>, Error> {
        let pgresults = self
            .fetch_pipeline(pipeline)
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

    pub async fn fetch_pipeline<'e, 'c: 'e, 'q: 'e, const N: usize>(
        &'c mut self,
        pipeline: PgExtendedQueryPipeline<'q, N>,
    ) -> Result<impl Stream<Item = Result<Either<PgQueryResult, PgRow>, Error>> + 'e, Error> {
        self.run_pipeline(pipeline).await
    }
}

// Private interface section

impl<'q, const N: usize> PgExtendedQueryPipeline<'q, N> {
    fn len(&self) -> usize {
        self.queries.len()
    }

    fn queries(&self) -> &SmallVec<[QueryContext<'q>; N]> {
        &self.queries
    }

    fn into_querycontext_stream(self) -> impl Stream<Item = QueryContext<'q>> {
        stream::iter(self.queries)
    }
}

impl PgConnection {
    async fn get_or_prepare_pipeline<'q, const N: usize>(
        &mut self,
        pipeline: PgExtendedQueryPipeline<'q, N>,
    ) -> Result<SmallVec<[(Oid, PgArguments); N]>, Error> {
        let prepared_statements = SmallVec::<[(Oid, PgArguments); N]>::new();

        pipeline
            .into_querycontext_stream()
            .map(|v| Ok(v))
            .try_fold(
                (self, prepared_statements),
                |(conn, mut prepared), (sql, maybe_arguments, persistent, maybe_metadata)| {
                    async move {
                        let mut arguments = maybe_arguments.unwrap_or_default();
                        // prepare the statement if this our first time executing it
                        // always return the statement ID here
                        let (statement, metadata) = conn
                            .get_or_prepare(sql, &arguments.types, persistent, maybe_metadata)
                            .await?;

                        // patch holes created during encoding
                        arguments.apply_patches(conn, &metadata.parameters).await?;

                        // apply patches use fetch_optional thaht may produce `PortalSuspended` message,
                        // consume messages til `ReadyForQuery` before bind and execute
                        conn.wait_until_ready().await?;
                        prepared.push((statement, arguments));
                        Ok((conn, prepared))
                    }
                },
            )
            .await
            .map(|(_, prepared)| prepared)
    }

    async fn run_pipeline<'e, 'c: 'e, 'q: 'e, const N: usize>(
        &'c mut self,
        pipeline: PgExtendedQueryPipeline<'q, N>,
    ) -> Result<impl Stream<Item = Result<Either<PgQueryResult, PgRow>, Error>> + 'e, Error> {
        // loggers stack is in reversed query order
        let mut loggers_stack = self.query_loggers_stack(&pipeline);

        // before we continue, wait until we are "ready" to accept more queries
        self.wait_until_ready().await?;

        let pipeline_length = pipeline.len();
        let prepared_statements = self.get_or_prepare_pipeline(pipeline).await?;

        prepared_statements
            .into_iter()
            .for_each(|(statement, arguments)| {
                // bind to attach the arguments to the statement and create a portal
                self.stream.write(Bind {
                    portal: None,
                    statement,
                    formats: &[PgValueFormat::Binary],
                    num_params: arguments.types.len() as i16,
                    params: &arguments.buffer,
                    result_formats: &[PgValueFormat::Binary],
                });

                self.stream.write(message::Execute {
                    portal: None,
                    // result set is expected to be small enough to buffer on client side
                    // don't use server-side cursors
                    limit: 0,
                });
            });

        // finally, [Sync] asks postgres to process the messages that we sent and respond with
        // a [ReadyForQuery] message when it's completely done.
        self.write_sync();
        // send all commands in batch
        self.stream.flush().await?;

        Ok(try_stream! {
            let mut metadata = Arc::new(PgStatementMetadata::default());
            // prepared statements are binary
            let format = PgValueFormat::Binary;

            loop {
                let message = self.stream.recv().await?;

                match message.format {
                    MessageFormat::BindComplete
                    | MessageFormat::ParseComplete
                    | MessageFormat::ParameterDescription
                    | MessageFormat::NoData => {
                        // harmless messages to ignore
                    }

                    MessageFormat::CommandComplete => {
                        // a SQL command completed normally
                        let cc: CommandComplete = message.decode()?;

                        let rows_affected = cc.rows_affected();
                        if let Some(logger) = loggers_stack.last_mut() {
                            logger.increase_rows_affected(rows_affected);
                            // drop and finish current logger
                            loggers_stack.pop();
                        }
                        else {
                            return Err(err_protocol!(
                                "execute: received more CommandComplete messages than expected; expected: {}",
                                pipeline_length
                            ));

                        }

                        r#yield!(Either::Left(PgQueryResult {
                            rows_affected,
                        }));
                    }

                    MessageFormat::EmptyQueryResponse => {
                        // empty query string passed to an unprepared execute
                    }

                    MessageFormat::RowDescription => {
                        // indicates that a *new* set of rows are about to be returned
                        let (columns, column_names) = self
                            .handle_row_description(Some(message.decode()?), false)
                            .await?;

                        metadata = Arc::new(PgStatementMetadata {
                            column_names,
                            columns,
                            parameters: Vec::default(),
                        });
                    }

                    MessageFormat::DataRow => {
                        if let Some(logger) = loggers_stack.last_mut() {
                            logger.increment_rows_returned();
                        }
                        else {
                            return Err(err_protocol!(
                                "execute: received a data row after receiving the expected {} CommandComplete messages",
                                pipeline_length
                            ));

                        }

                        // one of the set of rows returned by a SELECT, FETCH, etc query
                        let data: DataRow = message.decode()?;
                        let row = PgRow {
                            data,
                            format,
                            metadata: Arc::clone(&metadata),
                        };

                        r#yield!(Either::Right(row));
                    }

                    MessageFormat::ReadyForQuery => {
                        // processing of the query string is complete
                        self.handle_ready_for_query(message)?;
                        break;
                    }

                    _ => {
                        return Err(err_protocol!(
                            "execute: unexpected message: {:?}",
                            message.format
                        ));
                    }
                }
            }

            Ok(())
        })
    }

    fn query_loggers_stack<'q, const N: usize>(
        &self,
        pipeline: &PgExtendedQueryPipeline<'q, N>,
    ) -> SmallVec<[QueryLogger<'q>; N]> {
        pipeline
            .queries()
            .iter()
            .rev()
            .map(|(q_ref, _, _, _)| QueryLogger::new(*q_ref, self.log_settings.clone()))
            .collect()
    }
}
