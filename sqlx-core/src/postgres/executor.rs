use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_util::{stream, StreamExt, TryStreamExt};

use crate::arguments::Arguments;
use crate::cursor::Cursor;
use crate::describe::{Column, Describe};
use crate::executor::{Execute, Executor, RefExecutor};
use crate::postgres::protocol::{
    self, CommandComplete, Message, ParameterDescription, ReadyForQuery, RowDescription,
    StatementId, TypeFormat, TypeId,
};
use crate::postgres::row::Column as StatementColumn;
use crate::postgres::row::Statement;
use crate::postgres::type_info::SharedStr;
use crate::postgres::types::try_resolve_type_name;
use crate::postgres::{
    PgArguments, PgConnection, PgCursor, PgQueryAs, PgRow, PgTypeInfo, Postgres,
};
use crate::query_as::query_as;
use crate::row::Row;

impl PgConnection {
    pub(crate) fn write_simple_query(&mut self, query: &str) {
        self.stream.write(protocol::Query(query));
    }

    pub(crate) async fn write_prepare(
        &mut self,
        query: &str,
        args: &PgArguments,
    ) -> crate::Result<StatementId> {
        if let Some(&id) = self.cache_statement_id.get(query) {
            Ok(id)
        } else {
            let id = StatementId(self.next_statement_id);

            self.next_statement_id += 1;

            // Build a list of type OIDs from the type info array provided by PgArguments
            // This may need to query Postgres for an OID of a user-defined type

            let mut types = Vec::with_capacity(args.types.len());

            for ty in &args.types {
                types.push(if let Some(oid) = ty.id {
                    oid.0
                } else {
                    self.get_type_id_by_name(&*ty.name).await?
                });
            }

            self.stream.write(protocol::Parse {
                statement: id,
                param_types: &*types,
                query,
            });

            // [Describe] will return the expected result columns and types
            self.write_describe(protocol::Describe::Statement(id));
            self.write_sync();

            // Flush commands and handle ParseComplete and RowDescription
            self.wait_until_ready().await?;
            self.stream.flush().await?;
            self.is_ready = false;

            // wait for `ParseComplete`
            match self.stream.receive().await? {
                Message::ParseComplete => {}
                message => {
                    return Err(protocol_err!("run: unexpected message: {:?}", message).into());
                }
            }

            // expecting a `ParameterDescription` next
            let pd = self.expect_param_desc().await?;

            // expecting a `RowDescription` next (or `NoData` for an empty statement)
            let statement = self.expect_row_desc(pd).await?;

            // cache statement ID and statement description
            self.cache_statement_id.insert(query.into(), id);
            self.cache_statement.insert(id, Arc::new(statement));

            Ok(id)
        }
    }

    async fn parse_parameter_description(
        &mut self,
        pd: ParameterDescription,
    ) -> crate::Result<Box<[PgTypeInfo]>> {
        let mut params = Vec::with_capacity(pd.ids.len());

        for ty in pd.ids.iter() {
            let type_info = self.get_type_info_by_oid(ty.0, true).await?;

            params.push(type_info);
        }

        Ok(params.into_boxed_slice())
    }

    pub(crate) async fn parse_row_description(
        &mut self,
        mut rd: RowDescription,
        params: Box<[PgTypeInfo]>,
        type_format: Option<TypeFormat>,
        fetch_type_info: bool,
    ) -> crate::Result<Statement> {
        let mut names = HashMap::new();
        let mut columns = Vec::new();

        columns.reserve(rd.fields.len());
        names.reserve(rd.fields.len());

        for (index, field) in rd.fields.iter_mut().enumerate() {
            let name = if let Some(name) = field.name.take() {
                let name = SharedStr::from(name.into_string());
                names.insert(name.clone(), index);
                Some(name)
            } else {
                None
            };

            let type_info = self
                .get_type_info_by_oid(field.type_id.0, fetch_type_info)
                .await?;

            columns.push(StatementColumn {
                type_info,
                name,
                format: type_format.unwrap_or(field.type_format),
                table_id: field.table_id,
                column_id: field.column_id,
            });
        }

        Ok(Statement {
            params,
            columns: columns.into_boxed_slice(),
            names,
        })
    }

    async fn expect_param_desc(&mut self) -> crate::Result<ParameterDescription> {
        let description = match self.stream.receive().await? {
            Message::ParameterDescription => ParameterDescription::read(self.stream.buffer())?,

            message => {
                return Err(
                    protocol_err!("next/describe: unexpected message: {:?}", message).into(),
                );
            }
        };

        Ok(description)
    }

    // Used to describe the incoming results
    // We store the column map in an Arc and share it among all rows
    async fn expect_row_desc(&mut self, pd: ParameterDescription) -> crate::Result<Statement> {
        let description: Option<_> = match self.stream.receive().await? {
            Message::RowDescription => Some(RowDescription::read(self.stream.buffer())?),

            Message::NoData => None,

            message => {
                return Err(
                    protocol_err!("next/describe: unexpected message: {:?}", message).into(),
                );
            }
        };

        let params = self.parse_parameter_description(pd).await?;

        if let Some(description) = description {
            self.parse_row_description(description, params, Some(TypeFormat::Binary), true)
                .await
        } else {
            Ok(Statement {
                params,
                names: HashMap::new(),
                columns: Default::default(),
            })
        }
    }

    pub(crate) fn write_describe(&mut self, d: protocol::Describe) {
        self.stream.write(d);
    }

    pub(crate) async fn write_bind(
        &mut self,
        portal: &str,
        statement: StatementId,
        args: &mut PgArguments,
    ) -> crate::Result<()> {
        args.buffer.patch_type_holes(self).await?;

        self.stream.write(protocol::Bind {
            portal,
            statement,
            formats: &[TypeFormat::Binary],
            values_len: args.types.len() as i16,
            values: &*args.buffer,
            result_formats: &[TypeFormat::Binary],
        });

        Ok(())
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
                if let Message::ReadyForQuery = self.stream.receive().await? {
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
        let statement = if let Some(mut arguments) = arguments {
            // Check the statement cache for a statement ID that matches the given query
            // If it doesn't exist, we generate a new statement ID and write out [Parse] to the
            // connection command buffer
            let statement = self.write_prepare(query, &arguments).await?;

            // Next, [Bind] attaches the arguments to the statement and creates a named portal
            self.write_bind("", statement, &mut arguments).await?;

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

    async fn do_describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> crate::Result<Describe<Postgres>> {
        let statement_id = self.write_prepare(query, &Default::default()).await?;
        let statement = &self.cache_statement[&statement_id];
        let columns = statement.columns.to_vec();

        Ok(Describe {
            param_types: statement
                .params
                .iter()
                .map(|info| Some(info.clone()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            result_columns: self.map_result_columns(columns).await?.into_boxed_slice(),
        })
    }

    pub(crate) async fn get_type_id_by_name(&mut self, name: &str) -> crate::Result<u32> {
        if let Some(oid) = self.cache_type_oid.get(name) {
            return Ok(*oid);
        }

        // language=SQL
        let (oid,): (u32,) = query_as(
            "
SELECT oid FROM pg_catalog.pg_type WHERE typname ILIKE $1
                ",
        )
        .bind(name)
        .fetch_one(&mut *self)
        .await?;

        let shared = SharedStr::from(name.to_owned());

        self.cache_type_oid.insert(shared.clone(), oid);
        self.cache_type_name.insert(oid, shared.clone());

        Ok(oid)
    }

    pub(crate) async fn get_type_info_by_oid(
        &mut self,
        oid: u32,
        fetch_type_info: bool,
    ) -> crate::Result<PgTypeInfo> {
        if let Some(name) = try_resolve_type_name(oid) {
            return Ok(PgTypeInfo::new(TypeId(oid), name));
        }

        if let Some(name) = self.cache_type_name.get(&oid) {
            return Ok(PgTypeInfo::new(TypeId(oid), name));
        }

        let name = if fetch_type_info {
            // language=SQL
            let (name,): (String,) = query_as(
                "
    SELECT UPPER(typname) FROM pg_catalog.pg_type WHERE oid = $1
                    ",
            )
            .bind(oid)
            .fetch_one(&mut *self)
            .await?;

            // Emplace the new type name <-> OID association in the cache
            let shared = SharedStr::from(name);

            self.cache_type_oid.insert(shared.clone(), oid);
            self.cache_type_name.insert(oid, shared.clone());

            shared
        } else {
            // NOTE: The name isn't too important for the decode lifecycle of TEXT
            SharedStr::Static("")
        };

        Ok(PgTypeInfo::new(TypeId(oid), name))
    }

    async fn map_result_columns(
        &mut self,
        columns: Vec<StatementColumn>,
    ) -> crate::Result<Vec<Column<Postgres>>> {
        if columns.is_empty() {
            return Ok(vec![]);
        }

        let mut query = "select col.idx, pg_attribute.attnotnull from (VALUES ".to_string();
        let mut pushed = false;
        let mut args = PgArguments::default();

        for (i, (column, bind)) in columns.iter().zip((1..).step_by(3)).enumerate() {
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
            args.add(column.table_id.map(|id| id as i32));
            args.add(column.column_id);
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
            .zip(stream::iter(columns.into_iter().enumerate()))
            .map(|(row, (fidx, column))| -> crate::Result<Column<_>> {
                let (idx, non_null) = row?;

                if idx != fidx as i32 {
                    return Err(
                        protocol_err!("missing field from query, field: {:?}", column).into(),
                    );
                }

                Ok(Column {
                    name: column.name.map(|name| (&*name).into()),
                    table_id: column.table_id,
                    type_info: Some(column.type_info),
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
            match self.stream.receive().await? {
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
                    // TODO: How should we handle an ERROR status form ReadyForQuery
                    let _ready = ReadyForQuery::read(self.stream.buffer())?;

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

    fn execute<'e, 'q: 'e, 'c: 'e, E: 'e>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        log_execution!(query, {
            Box::pin(async move {
                let (query, arguments) = query.into_parts();

                self.run(query, arguments).await?;
                self.affected_rows().await
            })
        })
    }

    fn fetch<'q, E>(&mut self, query: E) -> PgCursor<'_, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        log_execution!(query, { PgCursor::from_connection(self, query) })
    }

    #[doc(hidden)]
    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move { self.do_describe(query.into_parts().0).await })
    }
}

impl<'c> RefExecutor<'c> for &'c mut super::PgConnection {
    type Database = Postgres;

    fn fetch_by_ref<'q, E>(self, query: E) -> PgCursor<'c, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        log_execution!(query, { PgCursor::from_connection(self, query) })
    }
}
