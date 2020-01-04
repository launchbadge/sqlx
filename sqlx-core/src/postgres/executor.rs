use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::describe::{Column, Describe};
use crate::postgres::protocol::{self, Encode, Message, StatementId};
use crate::postgres::types::TypeFormat;
use crate::postgres::{PgArguments, PgRow, Postgres};

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
            Some(Step::RowDesc(desc)) => desc,

            step => {
                return Err(protocol_err!("expected RowDescription; received {:?}", step).into());
            }
        };

        Ok(Describe {
            param_types: params.ids,
            result_columns: result
                .fields
                .into_vec()
                .into_iter()
                // TODO: Should [Column] just wrap [protocol::Field] ?
                .map(|field| Column {
                    name: field.name,
                    table_id: field.table_id,
                    type_id: field.type_id,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
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
