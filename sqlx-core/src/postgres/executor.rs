use futures_core::future::BoxFuture;

use crate::cursor::Cursor;
use crate::describe::{Column, Describe};
use crate::executor::{Execute, Executor, RefExecutor};
use crate::postgres::protocol::{
    self, CommandComplete, Message, ParameterDescription, RowDescription, StatementId, TypeFormat,
};
use crate::postgres::{PgArguments, PgConnection, PgCursor, PgTypeInfo, Postgres};

impl PgConnection {
    pub(crate) fn write_simple_query(&mut self, query: &str) {
        self.stream.write(protocol::Query(query));
    }

    pub(crate) fn write_prepare(&mut self, query: &str, args: &PgArguments) -> StatementId {
        if let Some(&id) = self.cache_statement.get(query) {
            id
        } else {
            let id = StatementId(self.next_statement_id);

            self.next_statement_id += 1;

            self.stream.write(protocol::Parse {
                statement: id,
                query,
                param_types: &*args.types,
            });

            self.cache_statement.insert(query.into(), id);

            id
        }
    }

    pub(crate) fn write_describe(&mut self, d: protocol::Describe) {
        self.stream.write(d);
    }

    pub(crate) fn write_bind(&mut self, portal: &str, statement: StatementId, args: &PgArguments) {
        self.stream.write(protocol::Bind {
            portal,
            statement,
            formats: &[TypeFormat::Binary],
            values_len: args.types.len() as i16,
            values: &*args.values,
            result_formats: &[TypeFormat::Binary],
        });
    }

    pub(crate) fn write_execute(&mut self, portal: &str, limit: i32) {
        self.stream.write(protocol::Execute { portal, limit });
    }

    pub(crate) fn write_sync(&mut self) {
        self.stream.write(protocol::Sync);
    }

    async fn wait_until_ready(&mut self) -> crate::Result<()> {
        // depending on how the previous query finished we may need to continue
        // pulling messages from the stream until we receive a [ReadyForQuery] message

        // postgres sends the [ReadyForQuery] message when it's fully complete with processing
        // the previous query

        if !self.is_ready {
            loop {
                if let Message::ReadyForQuery = self.stream.read().await? {
                    // we are now ready to go
                    self.is_ready = true;
                    break;
                }
            }
        }

        Ok(())
    }

    // Write out the query to the connection stream, ensure that we are synchronized at the
    // most recent [ReadyForQuery] and flush our buffer to postgres.
    //
    // It is safe to call this method repeatedly (but all data from postgres would be lost) but
    // it is assumed that a call to [PgConnection::affected_rows] or [PgCursor::next] would
    // immediately follow.
    pub(crate) async fn run(
        &mut self,
        query: &str,
        arguments: Option<PgArguments>,
    ) -> crate::Result<Option<StatementId>> {
        let statement = if let Some(arguments) = arguments {
            // Check the statement cache for a statement ID that matches the given query
            // If it doesn't exist, we generate a new statement ID and write out [Parse] to the
            // connection command buffer
            let statement = self.write_prepare(query, &arguments);

            // Next, [Bind] attaches the arguments to the statement and creates a named portal
            self.write_bind("", statement, &arguments);

            // Next, [Describe] will return the expected result columns and types
            // Conditionally run [Describe] only if the results have not been cached
            if !self.cache_statement_columns.contains_key(&statement) {
                self.write_describe(protocol::Describe::Portal(""));
            }

            // Next, [Execute] then executes the named portal
            self.write_execute("", 0);

            // Finally, [Sync] asks postgres to process the messages that we sent and respond with
            // a [ReadyForQuery] message when it's completely done. Theoretically, we could send
            // dozens of queries before a [Sync] and postgres can handle that. Execution on the server
            // is still serial but it would reduce round-trips. Some kind of builder pattern that is
            // termed batching might suit this.
            self.write_sync();

            Some(statement)
        } else {
            // https://www.postgresql.org/docs/12/protocol-flow.html#id-1.10.5.7.4
            self.write_simple_query(query);

            None
        };

        self.wait_until_ready().await?;

        self.stream.flush().await?;
        self.is_ready = false;

        Ok(statement)
    }

    async fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> crate::Result<Describe<Postgres>> {
        let statement = self.write_prepare(query, &Default::default());

        self.write_describe(protocol::Describe::Statement(statement));
        self.write_sync();

        self.stream.flush().await?;
        self.wait_until_ready().await?;

        let params = loop {
            match self.stream.read().await? {
                Message::ParseComplete => {}

                Message::ParameterDescription => {
                    break ParameterDescription::read(self.stream.buffer())?;
                }

                message => {
                    return Err(protocol_err!(
                        "expected ParameterDescription; received {:?}",
                        message
                    )
                    .into());
                }
            };
        };

        let result = match self.stream.read().await? {
            Message::NoData => None,
            Message::RowDescription => Some(RowDescription::read(self.stream.buffer())?),

            message => {
                return Err(protocol_err!(
                    "expected RowDescription or NoData; received {:?}",
                    message
                )
                .into());
            }
        };

        Ok(Describe {
            param_types: params
                .ids
                .iter()
                .map(|id| PgTypeInfo::new(*id))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            result_columns: result
                .map(|r| r.fields)
                .unwrap_or_default()
                .into_vec()
                .into_iter()
                // TODO: Should [Column] just wrap [protocol::Field] ?
                .map(|field| Column {
                    name: field.name,
                    table_id: field.table_id,
                    type_info: PgTypeInfo::new(field.type_id),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }

    // Poll messages from Postgres, counting the rows affected, until we finish the query
    // This must be called directly after a call to [PgConnection::execute]
    async fn affected_rows(&mut self) -> crate::Result<u64> {
        let mut rows = 0;

        loop {
            match self.stream.read().await? {
                Message::ParseComplete
                | Message::BindComplete
                | Message::NoData
                | Message::EmptyQueryResponse
                | Message::RowDescription => {}

                Message::DataRow => {
                    // TODO: should we log a warning? this is almost
                    //       definitely a programmer error
                }

                Message::CommandComplete => {
                    rows += CommandComplete::read(self.stream.buffer())?.affected_rows;
                }

                Message::ReadyForQuery => {
                    self.is_ready = true;
                    break;
                }

                message => {
                    return Err(
                        protocol_err!("affected_rows: unexpected message: {:?}", message).into(),
                    );
                }
            }
        }

        Ok(rows)
    }
}

impl Executor for super::PgConnection {
    type Database = Postgres;

    fn execute<'e, 'q, E: 'e>(&'e mut self, query: E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move {
            let (query, arguments) = query.into_parts();

            self.run(query, arguments).await?;
            self.affected_rows().await
        })
    }

    fn fetch<'q, E>(&mut self, query: E) -> PgCursor<'_, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        PgCursor::from_connection(self, query)
    }

    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move { self.describe(query.into_parts().0).await })
    }
}

impl<'c> RefExecutor<'c> for &'c mut super::PgConnection {
    type Database = Postgres;

    fn fetch_by_ref<'q, E>(self, query: E) -> PgCursor<'c, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        PgCursor::from_connection(self, query)
    }
}

impl_execute_for_query!(Postgres);
