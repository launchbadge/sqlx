use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use futures_core::future::BoxFuture;
use futures_util::{stream, StreamExt, TryStreamExt};

use crate::arguments::Arguments;
use crate::cursor::Cursor;
use crate::describe::{Column, Describe};
use crate::executor::{Execute, Executor, RefExecutor};
use crate::postgres::protocol::{
    self, CommandComplete, Field, Message, ParameterDescription, RowDescription, StatementId,
    TypeFormat, TypeId,
};
use crate::postgres::types::SharedStr;
use crate::postgres::{PgArguments, PgConnection, PgCursor, PgRow, PgTypeInfo, Postgres};
use crate::row::Row;

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
        self.is_ready = false;

        let statement = self.write_prepare(query, &Default::default());

        self.write_describe(protocol::Describe::Statement(statement));
        self.write_sync();

        self.stream.flush().await?;

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

        self.wait_until_ready().await?;

        let result_fields = result.map_or_else(Default::default, |r| r.fields);

        // TODO: cache this result
        let type_names = self
            .get_type_names(
                params
                    .ids
                    .iter()
                    .cloned()
                    .chain(result_fields.iter().map(|field| field.type_id)),
            )
            .await?;

        Ok(Describe {
            param_types: params
                .ids
                .iter()
                .map(|id| PgTypeInfo::new(*id, &type_names[&id.0]))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            result_columns: self
                .map_result_columns(result_fields, type_names)
                .await?
                .into_boxed_slice(),
        })
    }

    async fn get_type_names(
        &mut self,
        ids: impl IntoIterator<Item = TypeId>,
    ) -> crate::Result<HashMap<u32, SharedStr>> {
        let type_ids: HashSet<u32> = ids.into_iter().map(|id| id.0).collect::<HashSet<u32>>();

        if type_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // uppercase type names are easier to visually identify
        let mut query = "select types.type_id, UPPER(pg_type.typname) from (VALUES ".to_string();
        let mut args = PgArguments::default();
        let mut pushed = false;

        // TODO: dedup this with the one below, ideally as an API we can export
        for (i, (&type_id, bind)) in type_ids.iter().zip((1..).step_by(2)).enumerate() {
            if pushed {
                query += ", ";
            }

            pushed = true;
            let _ = write!(query, "(${}, ${})", bind, bind + 1);

            // not used in the output but ensures are values are sorted correctly
            args.add(i as i32);
            args.add(type_id as i32);
        }

        query += ") as types(idx, type_id) \
                  inner join pg_catalog.pg_type on pg_type.oid = type_id \
                  order by types.idx";

        crate::query::query(&query)
            .bind_all(args)
            .try_map(|row: PgRow| -> crate::Result<(u32, SharedStr)> {
                Ok((
                    row.try_get::<i32, _>(0)? as u32,
                    row.try_get::<String, _>(1)?.into(),
                ))
            })
            .fetch(self)
            .try_collect()
            .await
    }

    async fn map_result_columns(
        &mut self,
        fields: Box<[Field]>,
        type_names: HashMap<u32, SharedStr>,
    ) -> crate::Result<Vec<Column<Postgres>>> {
        if fields.is_empty() {
            return Ok(vec![]);
        }

        let mut query = "select col.idx, pg_attribute.attnotnull from (VALUES ".to_string();
        let mut pushed = false;
        let mut args = PgArguments::default();

        for (i, (field, bind)) in fields.iter().zip((1..).step_by(3)).enumerate() {
            if pushed {
                query += ", ";
            }

            pushed = true;
            let _ = write!(
                query,
                "(${}::int4, ${}::int4, ${}::int2)",
                bind,
                bind + 1,
                bind + 2
            );

            args.add(i as i32);
            args.add(field.table_id.map(|id| id as i32));
            args.add(field.column_id);
        }

        query += ") as col(idx, table_id, col_idx) \
        left join pg_catalog.pg_attribute on table_id is not null and attrelid = table_id and attnum = col_idx \
        order by col.idx;";

        log::trace!("describe pg_attribute query: {:#?}", query);

        crate::query::query(&query)
            .bind_all(args)
            .try_map(|row: PgRow| {
                let idx = row.try_get::<i32, _>(0)?;
                let non_null = row.try_get::<Option<bool>, _>(1)?;

                Ok((idx, non_null))
            })
            .fetch(self)
            .zip(stream::iter(fields.into_vec().into_iter().enumerate()))
            .map(|(row, (fidx, field))| -> crate::Result<Column<_>> {
                let (idx, non_null) = row?;

                if idx != fidx as i32 {
                    return Err(
                        protocol_err!("missing field from query, field: {:?}", field).into(),
                    );
                }

                Ok(Column {
                    name: field.name,
                    table_id: field.table_id,
                    type_info: PgTypeInfo::new(field.type_id, &type_names[&field.type_id.0]),
                    non_null,
                })
            })
            .try_collect()
            .await
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
