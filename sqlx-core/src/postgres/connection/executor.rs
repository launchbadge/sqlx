use async_stream::try_stream;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::{pin_mut, StreamExt, TryStreamExt};

use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::postgres::connection::describe::Statement;
use crate::postgres::message::{
    self, Bind, CommandComplete, DataRow, Describe, Flush, MessageFormat, ParameterDescription,
    Parse, Query, RowDescription,
};
use crate::postgres::{PgArguments, PgConnection, PgRow, PgValueFormat, Postgres};
use std::sync::Arc;

impl PgConnection {
    async fn prepare(
        &mut self,
        query: &str,
        arguments: &PgArguments,
    ) -> Result<Arc<Statement>, Error> {
        if let Some(statement) = self.cache_statement.get(query) {
            return Ok(statement.clone());
        }

        // unwrap: not possible, this is an infinite sequence
        let id = self.next_statement_id.next().unwrap();

        // build a list of type OIDs to send to the database in the PARSE command
        // we have not yet started the query sequence, so we are *safe* to cleanly make
        // additional queries here to get any missing OIDs

        let mut param_types = Vec::with_capacity(arguments.types.len());

        for ty in &arguments.types {
            param_types.push(if let Some(id) = ty.id {
                id
            } else {
                self.fetch_type_id_by_name(&*ty.name).await?
            });
        }

        // next we send the PARSE command to the server

        self.stream
            .write(Parse {
                param_types: &*param_types,
                query,
                statement: id,
            })
            .await?;

        // issue a DESCRIBE on the newly parsed statement
        // this will generate statement and parameter descriptions

        self.stream.write(Describe::Statement(id)).await?;

        // FLUSH should instruct the backend to send us the responses
        // for PARSE and DESCRIBE

        self.stream.write(Flush).await?;
        self.stream.flush().await?;

        // indicates that the SQL query string is now successfully parsed and has semantic validity
        let _: () = self
            .stream
            .recv_expect(MessageFormat::ParseComplete)
            .await?;

        // describes the parameters needed by the statement
        let params: ParameterDescription = self
            .stream
            .recv_expect(MessageFormat::ParameterDescription)
            .await?;

        let rows: Option<RowDescription> = match self.stream.recv().await? {
            // describes the rows that will be returned when the statement is eventually executed
            message if message.format == MessageFormat::RowDescription => Some(message.decode()?),

            // no data would be returned if this statement was executed
            message if message.format == MessageFormat::NoData => None,

            message => {
                return Err(err_protocol!(
                    "expecting RowDescription or NoData but received {:?}",
                    message.format
                ));
            }
        };

        let params = self.handle_parameter_description(params).await?;

        let rows = if let Some(rows) = rows {
            self.handle_row_description(rows, true).await?
        } else {
            Default::default()
        };

        let statement = Arc::new(Statement {
            id,
            param_types: Some(params),
            columns: rows.0,
            column_names: rows.1,
        });

        self.cache_statement
            .insert(query.to_owned(), statement.clone());

        Ok(statement)
    }

    async fn fetch_type_id_by_name(&mut self, name: &str) -> Result<u32, Error> {
        todo!("fetch_type_id_by_name")
    }

    async fn run(
        &mut self,
        query: &str,
        arguments: Option<PgArguments>,
        limit: u8,
    ) -> Result<impl Stream<Item = Result<Either<u64, PgRow>, Error>> + '_, Error> {
        // before we continue, wait until we are "ready" to accept more queries
        self.wait_until_ready().await?;

        let (mut statement, format) = if let Some(arguments) = arguments {
            // prepare the statement if this our first time executing it
            // always return the statement ID here
            let statement = self.prepare(query, &arguments).await?;

            // bind to attach the arguments to the statement and create a portal
            // TODO: args.buffer.patch_type_holes(self).await?;
            self.stream
                .write(Bind {
                    portal: None,
                    statement: statement.id,
                    formats: &[PgValueFormat::Binary],
                    num_params: arguments.types.len() as i16,
                    params: &*arguments.buffer,
                    result_formats: &[PgValueFormat::Binary],
                })
                .await?;

            // execute the portal
            self.stream
                .write(message::Execute {
                    portal: None,
                    limit: limit.into(),
                })
                .await?;

            // finally, [Sync] asks postgres to process the messages that we sent and respond with
            // a [ReadyForQuery] message when it's completely done. Theoretically, we could send
            // dozens of queries before a [Sync] and postgres can handle that. Execution on the server
            // is still serial but it would reduce round-trips. Some kind of builder pattern that is
            // termed batching might suit this.
            self.stream.write(message::Sync).await?;

            // prepared statements are binary
            (statement, PgValueFormat::Binary)
        } else {
            self.stream.write(Query(query)).await?;

            // and unprepared statements are text
            (Arc::new(Statement::empty()), PgValueFormat::Text)
        };

        // [Query] or [Sync] will trigger a [ReadyForQuery]
        self.pending_ready_for_query_count += 1;
        self.stream.flush().await?;

        Ok(try_stream! {
            loop {
                let message = self.stream.recv().await?;

                match message.format {
                    MessageFormat::BindComplete => {
                        // indicates that parameter binding was successful
                    }

                    MessageFormat::CommandComplete => {
                        // a SQL command completed normally
                        let cc: CommandComplete = message.decode()?;

                        yield Either::Left(cc.rows_affected());
                    }

                    MessageFormat::EmptyQueryResponse => {
                        // empty query string passed to an unprepared execute
                        statement = Arc::new(Statement::empty());
                    }

                    MessageFormat::RowDescription => {
                        // indicates that a *new* set of rows are about to be returned
                        // this message is only used here for the TEXT protocol
                        let rows = self
                            .handle_row_description(message.decode()?, false)
                            .await?;

                        statement = Arc::new(Statement {
                            id: 0,
                            column_names: rows.1,
                            param_types: None,
                            columns: rows.0,
                        });
                    }

                    MessageFormat::DataRow => {
                        // one of the set of rows returned by a SELECT, FETCH, etc query
                        let data: DataRow = message.decode()?;
                        let statement = Arc::clone(&statement);
                        let row = PgRow { data, format, statement };

                        yield Either::Right(row);
                    }

                    MessageFormat::ReadyForQuery => {
                        // processing of the query string is complete
                        self.handle_ready_for_query(message)?;
                        break;
                    }

                    _ => {
                        Err(err_protocol!(
                            "unexpected message: {:?}",
                            message.format
                        ))?;
                    }
                }
            }
        })
    }
}

impl<'c> Executor<'c> for &'c mut PgConnection {
    type Database = Postgres;

    fn fetch_many<'q: 'c, E>(self, mut query: E) -> BoxStream<'c, Result<Either<u64, PgRow>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream! {
            let s = self.run(s, arguments, 0).await?;
            pin_mut!(s);

            while let Some(s) = s.try_next().await? {
                yield s;
            }
        })
    }

    fn fetch_optional<'q: 'c, E>(self, mut query: E) -> BoxFuture<'c, Result<Option<PgRow>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(async move {
            let s = self.run(s, arguments, 2).await?;
            pin_mut!(s);

            let mut row = None;

            while let Some(s) = s.try_next().await? {
                if let Either::Right(r) = s {
                    if row.is_some() {
                        return Err(Error::FoundMoreThanOneRow);
                    }

                    row = Some(r);
                }
            }

            Ok(row)
        })
    }
}
