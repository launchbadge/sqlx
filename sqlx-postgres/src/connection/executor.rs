use crate::codec::backend::{CommandComplete, MessageFormat, ParameterDescription, RowDescription};
use crate::codec::frontend;
use crate::statement::StatementMetadata;
use crate::{PgColumn, PgConnection, PgTypeId, PgTypeInfo, Postgres};
use sqlx_core::error::Error;
use sqlx_core::execute::Execute;
use sqlx_core::to_value::ToValue;
use std::sync::Arc;

impl PgConnection {
    fn next_statement_id(&mut self) -> u32 {
        let id = self.next_statement_id;
        self.next_statement_id = self.next_statement_id.wrapping_add(1);

        id
    }

    async fn get_or_prepare<'q>(
        &mut self,
        sql: &'q str,
        arguments: &[&'q dyn ToValue<Postgres>],
    ) -> Result<Option<u32>, Error> {
        if let Some(statement) = self.cache_statement.get_mut(sql) {
            // statement is cached on this connection
            // we can skip [Parse]
            return Ok(Some(*statement));
        }

        let statement = if self.cache_statement.is_enabled() {
            Some(self.next_statement_id())
        } else {
            // prepared statements are completely disabled
            None
        };

        let mut parameter_types = Vec::with_capacity(arguments.len());

        for argument in arguments {
            parameter_types.push(match argument.produces() {
                PgTypeId::Oid(oid) => oid,
                PgTypeId::Name(_) => todo!("lazy bind name argument produces"),
            });
        }

        self.stream.write(frontend::Parse {
            statement,
            parameter_types: &parameter_types,
            query: sql,
        })?;

        self.stream.write(frontend::Sync)?;
        self.pending_ready_for_query += 1;

        self.recv_exact(MessageFormat::ParseComplete).await?;
        // if we do not return from this then the statement is never remembered to be parsed
        // as we have already incremented our [pending_ready_for_query] it should be safe to
        // directly call [prepare] again (after [drain])

        if let Some(statement) = statement {
            if let Some(statement) = self.cache_statement.insert(sql, statement) {
                self.stream.write(frontend::Close::Statement(statement))?;
                self.stream.write(frontend::Sync)?;
                self.pending_ready_for_query += 1;

                // we do not wait for CloseComplete or ReadyForQuery because its not
                // pathologically possible for this to fail. And this removes yield points
                // in an otherwise complex async method.
            }
        }

        // drain the stream of [ReadyForQuery] from either [Close] or [Parse] > [Sync]
        self.drain().await?;

        Ok(statement)
    }

    async fn get_or_describe<'q>(
        &mut self,
        sql: &'q str,
        statement: Option<u32>,
    ) -> Result<(Arc<StatementMetadata>, bool), Error> {
        if let Some(metadata) = self.cache_metadata.get_mut(sql) {
            // metadata is cached on this connection
            // we can skip [Describe]
            return Ok((Arc::clone(metadata), false));
        }

        self.stream
            .write(frontend::Describe::Statement(statement))?;

        self.stream.write(frontend::Sync)?;
        self.pending_ready_for_query += 1;

        let pd: ParameterDescription = self
            .recv_exact(MessageFormat::ParameterDescription)
            .await?
            .decode()?;

        // if we don't resume, we will drain [ParameterDescription] up until the next [ReadyForQuery]

        let rd: RowDescription = self
            .recv_exact(MessageFormat::RowDescription)
            .await?
            .decode()?;

        // if we don't resume, we will drain [RowDescription] up until the next [ReadyForQuery]

        let mut parameters = Vec::with_capacity(pd.types.len());

        for oid in pd.types {
            if let Some(ty) = PgTypeInfo::try_from_oid(oid) {
                parameters.push(ty);
            } else {
                todo!("non-builtin types for parameters")
            }
        }

        let mut columns = Vec::with_capacity(rd.fields.len());

        for field in &rd.fields {
            let type_info = if let Some(ty) = PgTypeInfo::try_from_oid(field.data_type_id) {
                ty
            } else {
                todo!("non-builtin types for results")
            };

            columns.push(PgColumn {
                name: field.name()?.to_owned(),
                type_info,
            })
        }

        let metadata = Arc::new(StatementMetadata {
            parameters,
            columns,
        });

        self.cache_metadata.insert(sql, Arc::clone(&metadata));

        // drain the stream of [ReadyForQuery] from [Describe] > [Sync]
        self.drain().await?;

        Ok((metadata, true))
    }

    pub(crate) async fn execute<'x, 'c: 'x, 'q: 'x, E: 'x + Execute<'q, Postgres>>(
        &'c mut self,
        mut query: E,
    ) -> Result<u64, Error> {
        // drain the connection buffer before attempting to use it for a *new* request
        self.drain().await?;

        let sql = query.sql();
        let arguments = query.take_arguments();

        if let Some(arguments) = arguments {
            // It would be more efficient to use *one* cache for both statements and metadata but
            // in order to allow our futures to be cancelled safely we need each dependent
            // operation to be atomic. It's the least expensive if we cache after we complete each
            // piece (prepare + describe) instead of at the end.

            // In addition, we want to allow the statement cache to be completely disabled,
            // in order to describe a potentially user-defined type from a [Describe] message,
            // we may need to issue a query to get the type information, this would overwrite
            // our "scratch space" for the prepared query. We can re-prepare the query in this
            // case and rely on our metadata cache to let us skip the [Describe] half.

            let statement = self.get_or_prepare(sql, &arguments).await?;
            // if [get_or_prepare] does not return, no path has been taken yet

            let (metadata, _) = self.get_or_describe(sql, statement).await?;
            // if [get_or_describe] does not return, we may have potentially prepared a statement
            // for no reason, but the connection will still be valid

            self.stream.write(frontend::Bind {
                statement,
                portal: None,
                formats: &[1],
                result_formats: &[1],
                parameters: &metadata.parameters,
                arguments: &arguments,
            })?;

            self.stream.write(frontend::Execute {
                limit: 1,
                portal: None,
            })?;

            self.stream.write(frontend::Sync)?;
        } else {
            self.stream.write(frontend::Query(sql))?;
        }

        // [Query] or [Sync] generates a [ReadyForQuery] message to
        // indicate request completion
        self.pending_ready_for_query += 1;

        let mut rows = 0_u64;

        loop {
            let message = self.recv().await?;
            // [recv] only returns whole messages; if it returns a message, that message will
            // not be returned again. we need to be careful to process it without using `await`

            match message.format {
                MessageFormat::CommandComplete => {
                    // a SQL command completed successfully
                    rows += message.decode::<CommandComplete>()?.rows_affected();
                }

                MessageFormat::EmptyQueryResponse => {
                    // the SQL query is an empty string
                    // this is returned instead of [CommandComplete]
                }

                MessageFormat::ReadyForQuery => {
                    // processing the entire SQL query is complete

                    // a separate message is sent to indicate this because the query string
                    // might contain multiple SQL commands

                    // [ReadyForQuery] is always sent, even in the case of an error

                    self.handle_ready_for_query(message.decode()?);
                    break;
                }

                message => {
                    // println!("unhandled: {:?}", message);
                }
            }
        }

        Ok(rows)
    }
}
