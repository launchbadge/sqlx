use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::{pin_mut, TryStreamExt};
use std::sync::Arc;

use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::postgres::message::{
    self, Bind, Close, CommandComplete, DataRow, Flush, MessageFormat, ParameterDescription, Parse,
    Query, RowDescription,
};
use crate::postgres::statement::Statement;
use crate::postgres::type_info::PgType;
use crate::postgres::{PgArguments, PgConnection, PgRow, PgValueFormat, Postgres};

async fn prepare(
    conn: &mut PgConnection,
    query: &str,
    arguments: &PgArguments,
) -> Result<Statement, Error> {
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
    conn.stream.write(Flush);
    conn.stream.flush().await?;

    // indicates that the SQL query string is now successfully parsed and has semantic validity
    let _: () = conn
        .stream
        .recv_expect(MessageFormat::ParseComplete)
        .await?;

    // describe the statement and, again, ask the server to immediately respond
    // we need to fully realize the types
    conn.stream.write(message::Describe::Statement(id));
    conn.stream.write(message::Flush);
    conn.stream.flush().await?;

    let params = recv_desc_params(conn).await?;
    let rows = recv_desc_rows(conn).await?;

    let params = conn.handle_parameter_description(params).await?;

    conn.handle_row_description(rows, true).await?;

    Ok(Statement::new(id, params))
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
    async fn prepare(
        &mut self,
        query: &str,
        arguments: &PgArguments,
    ) -> Result<&mut Statement, Error> {
        if self.cache_statement.contains_key(query) {
            return Ok(self.cache_statement.get_mut(query).unwrap());
        }

        let mut statement = prepare(self, query, arguments).await?;

        // While calling this recursively, we must end the recursion and skip
        // fetching column info for the describe queries.
        if !statement.columns_set() && !self.describe_mode {
            self.describe_mode = true;
            let columns = self.scratch_row_columns.clone();

            match self.map_result_columns(&columns).await {
                Ok(columns) => {
                    self.describe_mode = false;
                    statement.set_columns(columns);
                }
                Err(e) => {
                    self.describe_mode = false;
                    return Err(e);
                }
            }
        }

        if let Some(statement) = self.cache_statement.insert(query, statement) {
            self.stream.write(Close::Statement(statement.id()));
            self.stream.write(Flush);
            self.stream.flush().await?;
        }

        // We just inserted the statement above, we'll always have it in the
        // cache at this point.
        Ok(self.cache_statement.get_mut(query).unwrap())
    }

    async fn run(
        &mut self,
        query: &str,
        arguments: Option<PgArguments>,
        limit: u8,
    ) -> Result<impl Stream<Item = Result<Either<u64, PgRow>, Error>> + '_, Error> {
        // before we continue, wait until we are "ready" to accept more queries
        self.wait_until_ready().await?;

        let format = if let Some(mut arguments) = arguments {
            // prepare the statement if this our first time executing it
            // always return the statement ID here
            let statement = self.prepare(query, &arguments).await?;
            let statement_id = statement.id();

            // patch holes created during encoding
            arguments.buffer.patch_type_holes(self).await?;

            // describe the statement and, again, ask the server to immediately respond
            // we need to fully realize the types
            self.stream
                .write(message::Describe::Statement(statement_id));
            self.stream.write(message::Flush);
            self.stream.flush().await?;

            let _ = recv_desc_params(self).await?;
            let rows = recv_desc_rows(self).await?;

            self.handle_row_description(rows, true).await?;
            self.wait_until_ready().await?;

            // bind to attach the arguments to the statement and create a portal
            self.stream.write(Bind {
                portal: None,
                statement: statement_id,
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
            self.stream.write(message::Sync);

            // prepared statements are binary
            PgValueFormat::Binary
        } else {
            self.stream.write(Query(query));

            // and unprepared statements are text
            PgValueFormat::Text
        };

        // [Query] or [Sync] will trigger a [ReadyForQuery]
        self.pending_ready_for_query_count += 1;
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

                        r#yield!(Either::Left(cc.rows_affected()));
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
    ) -> BoxStream<'e, Result<Either<u64, PgRow>, Error>>
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

    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Describe<Postgres>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();

        Box::pin(async move {
            let statement = self.prepare(s, &Default::default()).await?;

            let params = statement.params().to_vec();
            let columns = statement.columns().to_vec();

            Ok(Describe { params, columns })
        })
    }
}
