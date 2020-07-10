use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::{pin_mut, TryStreamExt};
use std::{borrow::Cow, sync::Arc};

use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::postgres::message::{
    self, Bind, Close, CommandComplete, DataRow, MessageFormat, ParameterDescription, Parse, Query,
    RowDescription,
};
use crate::postgres::type_info::PgType;
use crate::postgres::{
    statement::PgStatement, PgArguments, PgConnection, PgDone, PgRow, PgValueFormat, Postgres,
};
use crate::statement::StatementInfo;
use message::Flush;

async fn prepare(
    conn: &mut PgConnection,
    query: &str,
    arguments: &PgArguments,
) -> Result<PgStatement, Error> {
    // before we continue, wait until we are "ready" to accept more queries
    conn.wait_until_ready().await?;

    let id = conn.next_statement_id;
    conn.next_statement_id = conn.next_statement_id.wrapping_add(1);

    // build a list of type OIDs to send to the database in the PARSE command
    // we have not yet started the query sequence, so we are *safe* to cleanly make
    // additional queries here to get any missing OIDs

    let mut param_types = Vec::with_capacity(arguments.types.len());
    let mut has_fetched = false;

    for ty in &arguments.types {
        param_types.push(if let PgType::DeclareWithName(name) = &ty.0 {
            has_fetched = true;
            conn.fetch_type_id_by_name(name).await?
        } else {
            ty.0.oid()
        });
    }

    // flush and wait until we are re-ready
    if has_fetched {
        conn.wait_until_ready().await?;
    }

    // next we send the PARSE command to the server
    conn.stream.write(Parse {
        param_types: &*param_types,
        query,
        statement: id,
    });

    // we ask for the server to immediately send us the result of the PARSE command by using FLUSH

    conn.write_sync();
    conn.stream.flush().await?;

    // indicates that the SQL query string is now successfully parsed and has semantic validity
    let _: () = conn
        .stream
        .recv_expect(MessageFormat::ParseComplete)
        .await?;

    conn.recv_ready_for_query().await?;

    // get the statement columns and parameters
    conn.stream.write(message::Describe::Statement(id));
    conn.write_sync();

    conn.stream.flush().await?;

    let parameters = recv_desc_params(conn).await?;
    let rows = recv_desc_rows(conn).await?;

    conn.recv_ready_for_query().await?;

    let parameters = conn.handle_parameter_description(parameters).await?;

    conn.handle_row_description(rows, true).await?;

    let columns = (&*conn.scratch_row_columns).clone();

    Ok(PgStatement {
        id,
        parameters,
        columns,
    })
}

async fn recv_desc_params(conn: &mut PgConnection) -> Result<ParameterDescription, Error> {
    conn.stream
        .recv_expect(MessageFormat::ParameterDescription)
        .await
}

async fn recv_desc_rows(conn: &mut PgConnection) -> Result<Option<RowDescription>, Error> {
    let rows: Option<RowDescription> = match conn.stream.recv().await? {
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

    Ok(rows)
}

impl PgConnection {
    // wait for CloseComplete to indicate a statement was closed
    pub(super) async fn wait_for_close_complete(&mut self, mut count: usize) -> Result<(), Error> {
        // we need to wait for the [CloseComplete] to be returned from the server
        while count > 0 {
            match self.stream.recv().await? {
                message if message.format == MessageFormat::PortalSuspended => {
                    // there was an open portal
                    // this can happen if the last time a statement was used it was not fully executed
                    // such as in [fetch_one]
                }

                message if message.format == MessageFormat::CloseComplete => {
                    // successfully closed the statement (and freed up the server resources)
                    count -= 1;
                }

                message => {
                    return Err(err_protocol!(
                        "expecting PortalSuspended or CloseComplete but received {:?}",
                        message.format
                    ));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn write_sync(&mut self) {
        self.stream.write(message::Sync);

        // all SYNC messages will return a ReadyForQuery
        self.pending_ready_for_query_count += 1;
    }

    async fn prepare<'a>(
        &'a mut self,
        query: &str,
        arguments: &PgArguments,
        store_to_cache: bool,
    ) -> Result<Cow<'a, PgStatement>, Error> {
        let contains = self.cache_statement.contains_key(query);

        if contains {
            return Ok(Cow::Borrowed(
                &*self.cache_statement.get_mut(query).unwrap(),
            ));
        }

        let statement = prepare(self, query, arguments).await?;

        if store_to_cache && self.cache_statement.is_enabled() {
            if let Some(statement) = self.cache_statement.insert(query, statement) {
                self.stream.write(Close::Statement(statement.id));
                self.stream.write(Flush);

                self.stream.flush().await?;

                self.wait_for_close_complete(1).await?;
            }

            Ok(Cow::Borrowed(
                &*self.cache_statement.get_mut(query).unwrap(),
            ))
        } else {
            Ok(Cow::Owned(statement))
        }
    }

    async fn run(
        &mut self,
        query: &str,
        arguments: Option<PgArguments>,
        limit: u8,
    ) -> Result<impl Stream<Item = Result<Either<PgDone, PgRow>, Error>> + '_, Error> {
        // before we continue, wait until we are "ready" to accept more queries
        self.wait_until_ready().await?;

        let format = if let Some(mut arguments) = arguments {
            // prepare the statement if this our first time executing it
            // always return the statement ID here
            let statement = self.prepare(query, &arguments, true).await?.id;

            // patch holes created during encoding
            arguments.buffer.patch_type_holes(self).await?;

            // describe the statement and, again, ask the server to immediately respond
            // we need to fully realize the types
            self.stream.write(message::Describe::Statement(statement));

            self.write_sync();
            self.stream.flush().await?;

            let _ = recv_desc_params(self).await?;
            let rows = recv_desc_rows(self).await?;

            self.handle_row_description(rows, true).await?;
            self.wait_until_ready().await?;

            // bind to attach the arguments to the statement and create a portal
            self.stream.write(Bind {
                portal: None,
                statement,
                formats: &[PgValueFormat::Binary],
                num_params: arguments.types.len() as i16,
                params: &*arguments.buffer,
                result_formats: &[PgValueFormat::Binary],
            });

            // executes the portal up to the passed limit
            // the protocol-level limit acts nearly identically to the `LIMIT` in SQL
            self.stream.write(message::Execute {
                portal: None,
                limit: limit.into(),
            });

            // finally, [Sync] asks postgres to process the messages that we sent and respond with
            // a [ReadyForQuery] message when it's completely done. Theoretically, we could send
            // dozens of queries before a [Sync] and postgres can handle that. Execution on the server
            // is still serial but it would reduce round-trips. Some kind of builder pattern that is
            // termed batching might suit this.
            self.write_sync();

            // prepared statements are binary
            PgValueFormat::Binary
        } else {
            // Query will trigger a ReadyForQuery
            self.stream.write(Query(query));
            self.pending_ready_for_query_count += 1;

            // and unprepared statements are text
            PgValueFormat::Text
        };

        self.stream.flush().await?;

        Ok(try_stream! {
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

                        r#yield!(Either::Left(PgDone {
                            rows_affected: cc.rows_affected(),
                        }));
                    }

                    MessageFormat::EmptyQueryResponse => {
                        // empty query string passed to an unprepared execute
                    }

                    MessageFormat::RowDescription => {
                        // indicates that a *new* set of rows are about to be returned
                        self
                            .handle_row_description(Some(message.decode()?), false)
                            .await?;
                    }

                    MessageFormat::DataRow => {
                        // one of the set of rows returned by a SELECT, FETCH, etc query
                        let data: DataRow = message.decode()?;
                        let row = PgRow {
                            data,
                            format,
                            columns: Arc::clone(&self.scratch_row_columns),
                            column_names: Arc::clone(&self.scratch_row_column_names),
                        };

                        r#yield!(Either::Right(row));
                    }

                    MessageFormat::ReadyForQuery => {
                        // processing of the query string is complete
                        self.handle_ready_for_query(message)?;
                        break;
                    }

                    _ => {
                        Err(err_protocol!(
                            "execute: unexpected message: {:?}",
                            message.format
                        ))?;
                    }
                }
            }

            Ok(())
        })
    }
}

impl<'c> Executor<'c> for &'c mut PgConnection {
    type Database = Postgres;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<PgDone, PgRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream! {
            let s = self.run(s, arguments, 0).await?;
            pin_mut!(s);

            while let Some(v) = s.try_next().await? {
                r#yield!(v);
            }

            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<PgRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(async move {
            let s = self.run(s, arguments, 1).await?;
            pin_mut!(s);

            while let Some(s) = s.try_next().await? {
                if let Either::Right(r) = s {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<StatementInfo<Postgres>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();

        Box::pin(async move {
            let statement = self.prepare(s, &Default::default(), false).await?;
            let columns = statement.columns.clone();
            let params = statement.parameters.clone();
            let nullable = Vec::new();

            Ok(StatementInfo {
                columns,
                nullable,
                parameters: Some(Either::Left(params)),
            })
        })
    }

    fn describe_full<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<StatementInfo<Self::Database>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();

        Box::pin(async move {
            let statement = self.prepare(s, &Default::default(), false).await?;
            let columns = statement.columns.clone();
            let params = statement.parameters.clone();
            let nullable = self.get_nullable_for_columns(&columns).await?;

            Ok(StatementInfo {
                columns,
                nullable,
                parameters: Some(Either::Left(params)),
            })
        })
    }
}
