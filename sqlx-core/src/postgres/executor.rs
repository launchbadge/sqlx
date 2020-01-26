use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::io;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{stream, FutureExt, StreamExt, TryStreamExt};

use crate::arguments::Arguments;
use crate::describe::{Column, Describe, Nullability};
use crate::encode::IsNull::No;
use crate::postgres::{PgArguments, PgRow, PgTypeInfo, Postgres};
use crate::postgres::protocol::{self, Encode, Field, Message, StatementId, TypeFormat, TypeId};
use crate::row::Row;
use crate::postgres::types::SharedStr;

#[derive(Debug)]
enum Step {
    Command(u64),
    NoData,
    Row(protocol::DataRow),
    ParamDesc(Box<protocol::ParameterDescription>),
    RowDesc(Box<protocol::RowDescription>),
}

impl super::PgConnection {
    fn write_prepare(&mut self, query: &str, args: &PgArguments) -> StatementId {
        if let Some(&id) = self.statement_cache.get(query) {
            id
        } else {
            let id = StatementId(self.next_statement_id);
            self.next_statement_id += 1;

            protocol::Parse {
                statement: id,
                query,
                param_types: &*args.types,
            }
                .encode(self.stream.buffer_mut());

            self.statement_cache.put(query.to_owned(), id);

            id
        }
    }

    fn write_describe(&mut self, d: protocol::Describe) {
        d.encode(self.stream.buffer_mut())
    }

    fn write_bind(&mut self, portal: &str, statement: StatementId, args: &PgArguments) {
        protocol::Bind {
            portal,
            statement,
            formats: &[TypeFormat::Binary],
            // TODO: Early error if there is more than i16
            values_len: args.types.len() as i16,
            values: &*args.values,
            result_formats: &[TypeFormat::Binary],
        }
            .encode(self.stream.buffer_mut());
    }

    fn write_execute(&mut self, portal: &str, limit: i32) {
        protocol::Execute { portal, limit }.encode(self.stream.buffer_mut());
    }

    fn write_sync(&mut self) {
        protocol::Sync.encode(self.stream.buffer_mut());
    }

    async fn wait_until_ready(&mut self) -> crate::Result<()> {
        if !self.ready {
            while let Some(message) = self.receive().await? {
                match message {
                    Message::ReadyForQuery(_) => {
                        self.ready = true;
                        break;
                    }

                    _ => {
                        // Drain the stream
                    }
                }
            }
        }

        Ok(())
    }

    async fn step(&mut self) -> crate::Result<Option<Step>> {
        while let Some(message) = self.receive().await? {
            match message {
                Message::BindComplete
                | Message::ParseComplete
                | Message::PortalSuspended
                | Message::CloseComplete => {}

                Message::CommandComplete(body) => {
                    return Ok(Some(Step::Command(body.affected_rows)));
                }

                Message::NoData => {
                    return Ok(Some(Step::NoData));
                }

                Message::DataRow(body) => {
                    return Ok(Some(Step::Row(body)));
                }

                Message::ReadyForQuery(_) => {
                    self.ready = true;

                    return Ok(None);
                }

                Message::ParameterDescription(desc) => {
                    return Ok(Some(Step::ParamDesc(desc)));
                }

                Message::RowDescription(desc) => {
                    return Ok(Some(Step::RowDesc(desc)));
                }

                message => {
                    return Err(protocol_err!("received unexpected message: {:?}", message).into());
                }
            }
        }

        // Connection was (unexpectedly) closed
        Err(io::Error::from(io::ErrorKind::ConnectionAborted).into())
    }
}

impl super::PgConnection {
    async fn send<'e, 'q: 'e>(&'e mut self, command: &'q str) -> crate::Result<()> {
        protocol::Query(command).encode(self.stream.buffer_mut());

        self.wait_until_ready().await?;

        self.stream.flush().await?;
        self.ready = false;

        while let Some(_step) = self.step().await? {
            // Drain the stream until ReadyForQuery
        }

        Ok(())
    }

    async fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: PgArguments,
    ) -> crate::Result<u64> {
        let statement = self.write_prepare(query, &args);

        self.write_bind("", statement, &args);
        self.write_execute("", 1);
        self.write_sync();

        self.wait_until_ready().await?;

        self.stream.flush().await?;
        self.ready = false;

        let mut affected = 0;

        while let Some(step) = self.step().await? {
            if let Step::Command(cnt) = step {
                affected = cnt;
            }
        }

        Ok(affected)
    }

    // Initial part of [fetch]; write message to stream
    fn write_fetch(&mut self, query: &str, args: &PgArguments) -> StatementId {
        let statement = self.write_prepare(query, &args);

        self.write_bind("", statement, &args);

        if !self.statement_cache.has_columns(statement) {
            self.write_describe(protocol::Describe::Portal(""));
        }

        self.write_execute("", 0);
        self.write_sync();

        statement
    }

    async fn get_columns(
        &mut self,
        statement: StatementId,
    ) -> crate::Result<Arc<HashMap<Box<str>, usize>>> {
        if !self.statement_cache.has_columns(statement) {
            let desc: Option<_> = 'outer: loop {
                while let Some(step) = self.step().await? {
                    match step {
                        Step::RowDesc(desc) => break 'outer Some(desc),

                        Step::NoData => break 'outer None,

                        _ => {}
                    }
                }

                unreachable!();
            };

            let mut columns = HashMap::new();

            if let Some(desc) = desc {
                columns.reserve(desc.fields.len());

                for (index, field) in desc.fields.iter().enumerate() {
                    if let Some(name) = &field.name {
                        columns.insert(name.clone(), index);
                    }
                }
            }

            self.statement_cache.put_columns(statement, columns);
        }

        Ok(self.statement_cache.get_columns(statement))
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: PgArguments,
    ) -> BoxStream<'e, crate::Result<PgRow>> {
        Box::pin(async_stream::try_stream! {
            let statement = self.write_fetch(query, &args);

            self.wait_until_ready().await?;

            self.stream.flush().await?;
            self.ready = false;

            let columns = self.get_columns(statement).await?;

            while let Some(step) = self.step().await? {
                if let Step::Row(data) = step {
                    yield PgRow { data, columns: Arc::clone(&columns) };
                }
            }

            // No more rows in the result set
        })
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

        let params = match self.step().await? {
            Some(Step::ParamDesc(desc)) => desc,

            step => {
                return Err(
                    protocol_err!("expected ParameterDescription; received {:?}", step).into(),
                );
            }
        };

        let result = match self.step().await? {
            Some(Step::RowDesc(desc)) => Some(desc),
            Some(Step::NoData) => None,

            step => {
                return Err(protocol_err!("expected RowDescription; received {:?}", step).into());
            }
        };

        while let Some(_) = self.step().await? {}

        let result_fields = result.map_or_else(Default::default, |r| r.fields);

        // TODO: cache this result
        let type_names = self.get_type_names(
            params
                .ids
                .iter()
                .cloned()
                .chain(result_fields.iter().map(|field| field.type_id))
        )
            .await?;

        Ok(Describe {
            param_types: params
                .ids
                .iter()
                .map(|id| PgTypeInfo::new(*id, &type_names[&id.0]))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            result_columns: self.map_result_columns(result_fields, type_names).await?
                .into_boxed_slice(),
        })
    }

    async fn get_type_names(&mut self, ids: impl IntoIterator<Item = TypeId>) -> crate::Result<HashMap<u32, SharedStr>> {
        let type_ids: HashSet<u32> = ids.into_iter().map(|id| id.0).collect::<HashSet<u32>>();

        let mut query = "select types.type_id, pg_type.typname from (VALUES ".to_string();
        let mut args = PgArguments::default();
        let mut pushed = false;

        // TODO: dedup this with the one below, ideally as an API we can export
        for (i, (&type_id, bind)) in type_ids.iter().zip((1 .. ).step_by(2)).enumerate() {
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

        self.fetch(&query, args)
            .map_ok(|row: PgRow| -> (u32, SharedStr) {
                (row.get::<i32, _>(0) as u32, row.get::<String, _>(1).into())
            })
            .try_collect()
            .await
    }

    async fn map_result_columns(&mut self, fields: Box<[Field]>, type_names: HashMap<u32, SharedStr>) -> crate::Result<Vec<Column<Postgres>>> {
        use crate::describe::Nullability::*;

        if fields.is_empty() { return Ok(vec![]); }

        let mut query = "select col.idx, pg_attribute.attnotnull from (VALUES ".to_string();
        let mut pushed = false;
        let mut args = PgArguments::default();

        for (i, (field, bind)) in fields.iter().zip((1 ..).step_by(3)).enumerate() {
            if pushed {
                query += ", ";
            }

            pushed = true;
            let _ = write!(query, "(${}, ${}, ${})", bind, bind + 1, bind + 2);

            args.add(i as i32);
            args.add(field.table_id.map(|id| id as i32));
            args.add(field.column_id);
        }

        query += ") as col(idx, table_id, col_idx) \
        left join pg_catalog.pg_attribute on table_id is not null and attrelid = table_id and attnum = col_idx \
        order by col.idx;";

        log::trace!("describe pg_attribute query: {:#?}", query);

        self.fetch(&query, args)
            .zip(stream::iter(fields.into_vec().into_iter().enumerate()))
            .map(|(row, (fidx, field))| -> crate::Result<Column<_>> {
                let row = row?;
                let idx = row.get::<i32, _>(0);
                let nonnull = row.get::<Option<bool>, _>(1);

                if idx != fidx as i32 {
                    return Err(protocol_err!("missing field from query, field: {:?}", field).into());
                }

                Ok(Column {
                    name: field.name,
                    table_id: field.table_id,
                    type_info: PgTypeInfo::new(field.type_id, &type_names[&field.type_id.0]),
                    nullability: nonnull.map(|nonnull| if nonnull { NonNull } else { Nullable })
                        .unwrap_or(Unknown),
                })
            })
            .try_collect()
            .await
    }
}

impl crate::Executor for super::PgConnection {
    type Database = super::Postgres;

    fn send<'e, 'q: 'e>(&'e mut self, query: &'q str) -> BoxFuture<'e, crate::Result<()>> {
        Box::pin(self.send(query))
    }

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: PgArguments,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        Box::pin(self.execute(query, args))
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: PgArguments,
    ) -> BoxStream<'e, crate::Result<PgRow>> {
        self.fetch(query, args)
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>> {
        Box::pin(self.describe(query))
    }
}
