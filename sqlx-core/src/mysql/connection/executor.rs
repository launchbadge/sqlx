use std::sync::Arc;

use bytes::Bytes;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::{pin_mut, TryStreamExt};

use crate::describe::{Column, Describe};
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::ext::ustr::UStr;
use crate::mysql::connection::stream::Busy;
use crate::mysql::io::MySqlBufExt;
use crate::mysql::protocol::response::Status;
use crate::mysql::protocol::statement::{
    BinaryRow, Execute as StatementExecute, Prepare, PrepareOk, StmtClose,
};
use crate::mysql::protocol::text::{ColumnDefinition, ColumnFlags, Query, TextRow};
use crate::mysql::protocol::Packet;
use crate::mysql::row::MySqlColumn;
use crate::mysql::{
    MySql, MySqlArguments, MySqlConnection, MySqlRow, MySqlTypeInfo, MySqlValueFormat,
};

impl MySqlConnection {
    async fn prepare(&mut self, query: &str) -> Result<u32, Error> {
        if let Some(statement) = self.cache_statement.get_mut(query) {
            return Ok(*statement);
        }

        // https://dev.mysql.com/doc/internals/en/com-stmt-prepare.html
        // https://dev.mysql.com/doc/internals/en/com-stmt-prepare-response.html#packet-COM_STMT_PREPARE_OK

        self.stream.send_packet(Prepare { query }).await?;

        let ok: PrepareOk = self.stream.recv().await?;

        // the parameter definitions are very unreliable so we skip over them
        // as we have little use

        if ok.params > 0 {
            for _ in 0..ok.params {
                let _def: ColumnDefinition = self.stream.recv().await?;
            }

            self.stream.maybe_recv_eof().await?;
        }

        // the column definitions are berefit the type information from the
        // to-be-bound parameters; we will receive the output column definitions
        // once more on execute so we wait for that

        if ok.columns > 0 {
            for _ in 0..(ok.columns as usize) {
                let _def: ColumnDefinition = self.stream.recv().await?;
            }

            self.stream.maybe_recv_eof().await?;
        }

        // in case of the cache being full, close the least recently used statement
        if let Some(statement) = self.cache_statement.insert(query, ok.statement_id) {
            self.stream.send_packet(StmtClose { statement }).await?;
        }

        Ok(ok.statement_id)
    }

    async fn recv_result_metadata(&mut self, mut packet: Packet<Bytes>) -> Result<(), Error> {
        let num_columns: u64 = packet.get_uint_lenenc(); // column count

        // the result-set metadata is primarily a listing of each output
        // column in the result-set

        let column_names = Arc::make_mut(&mut self.scratch_row_column_names);
        let columns = Arc::make_mut(&mut self.scratch_row_columns);

        columns.clear();
        column_names.clear();

        for i in 0..num_columns {
            let def: ColumnDefinition = self.stream.recv().await?;

            let name = (match (def.name()?, def.alias()?) {
                (_, alias) if !alias.is_empty() => Some(alias),

                (name, _) if !name.is_empty() => Some(name),

                _ => None,
            })
            .map(UStr::new);

            if let Some(name) = &name {
                column_names.insert(name.clone(), i as usize);
            }

            let type_info = MySqlTypeInfo::from_column(&def);

            columns.push(MySqlColumn { name, type_info });
        }

        self.stream.maybe_recv_eof().await?;

        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    async fn run<'c>(
        &'c mut self,
        query: &str,
        arguments: Option<MySqlArguments>,
    ) -> Result<impl Stream<Item = Result<Either<u64, MySqlRow>, Error>> + 'c, Error> {
        self.stream.wait_until_ready().await?;
        self.stream.busy = Busy::Result;

        let format = if let Some(arguments) = arguments {
            let statement = self.prepare(query).await?;

            // https://dev.mysql.com/doc/internals/en/com-stmt-execute.html
            self.stream
                .send_packet(StatementExecute {
                    statement,
                    arguments: &arguments,
                })
                .await?;

            MySqlValueFormat::Binary
        } else {
            // https://dev.mysql.com/doc/internals/en/com-query.html
            self.stream.send_packet(Query(query)).await?;

            MySqlValueFormat::Text
        };

        Ok(Box::pin(try_stream! {
            loop {
                // query response is a meta-packet which may be one of:
                //  Ok, Err, ResultSet, or (unhandled) LocalInfileRequest
                let packet = self.stream.recv_packet().await?;

                if packet[0] == 0x00 || packet[0] == 0xff {
                    // first packet in a query response is OK or ERR
                    // this indicates either a successful query with no rows at all or a failed query
                    let ok = packet.ok()?;

                    r#yield!(Either::Left(ok.affected_rows));

                    if ok.status.contains(Status::SERVER_MORE_RESULTS_EXISTS) {
                        // more result sets exist, continue to the next one
                        continue;
                    }

                    self.stream.busy = Busy::NotBusy;
                    return Ok(());
                }

                // otherwise, this first packet is the start of the result-set metadata,
                self.stream.busy = Busy::Row;
                self.recv_result_metadata(packet).await?;

                // finally, there will be none or many result-rows
                loop {
                    let packet = self.stream.recv_packet().await?;

                    if packet[0] == 0xfe && packet.len() < 9 {
                        let eof = packet.eof(self.stream.capabilities)?;
                        r#yield!(Either::Left(0));

                        if eof.status.contains(Status::SERVER_MORE_RESULTS_EXISTS) {
                            // more result sets exist, continue to the next one
                            self.stream.busy = Busy::Result;
                            break;
                        }

                        self.stream.busy = Busy::NotBusy;
                        return Ok(());
                    }

                    let row = match format {
                        MySqlValueFormat::Binary => packet.decode_with::<BinaryRow, _>(&self.scratch_row_columns)?.0,
                        MySqlValueFormat::Text => packet.decode_with::<TextRow, _>(&self.scratch_row_columns)?.0,
                    };

                    let v = Either::Right(MySqlRow {
                        row,
                        format,
                        columns: Arc::clone(&self.scratch_row_columns),
                        column_names: Arc::clone(&self.scratch_row_column_names),
                    });

                    r#yield!(v);
                }
            }
        }))
    }
}

impl<'c> Executor<'c> for &'c mut MySqlConnection {
    type Database = MySql;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<u64, MySqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream! {
            let s = self.run(s, arguments).await?;
            pin_mut!(s);

            while let Some(v) = s.try_next().await? {
                r#yield!(v);
            }

            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<MySqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);

        Box::pin(async move {
            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxFuture<'e, Result<Describe<MySql>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let query = query.query();

        Box::pin(async move {
            self.stream.send_packet(Prepare { query }).await?;

            let ok: PrepareOk = self.stream.recv().await?;

            let mut params = Vec::with_capacity(ok.params as usize);
            let mut columns = Vec::with_capacity(ok.columns as usize);

            if ok.params > 0 {
                for _ in 0..ok.params {
                    let def: ColumnDefinition = self.stream.recv().await?;

                    params.push(MySqlTypeInfo::from_column(&def));
                }

                self.stream.maybe_recv_eof().await?;
            }

            // the column definitions are berefit the type information from the
            // to-be-bound parameters; we will receive the output column definitions
            // once more on execute so we wait for that

            if ok.columns > 0 {
                for _ in 0..(ok.columns as usize) {
                    let def: ColumnDefinition = self.stream.recv().await?;
                    let ty = MySqlTypeInfo::from_column(&def);
                    let alias = def.alias()?;

                    columns.push(Column {
                        name: if alias.is_empty() { def.name()? } else { alias }.to_owned(),
                        type_info: ty,
                        not_null: Some(def.flags.contains(ColumnFlags::NOT_NULL)),
                    })
                }

                self.stream.maybe_recv_eof().await?;
            }

            Ok(Describe { params, columns })
        })
    }
}
