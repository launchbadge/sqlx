use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::describe::{Column, Describe};
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::mssql::protocol::col_meta_data::Flags;
use crate::mssql::protocol::done::{Done, Status};
use crate::mssql::protocol::message::Message;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::rpc::{OptionFlags, Procedure, RpcRequest};
use crate::mssql::protocol::sql_batch::SqlBatch;
use crate::mssql::{Mssql, MssqlArguments, MssqlConnection, MssqlRow, MssqlTypeInfo};

impl MssqlConnection {
    pub(crate) async fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.stream.wbuf.is_empty() {
            self.pending_done_count += 1;
            self.stream.flush().await?;
        }

        while self.pending_done_count > 0 {
            let message = self.stream.recv_message().await?;

            if let Message::DoneProc(done) | Message::Done(done) = message {
                if !done.status.contains(Status::DONE_MORE) {
                    // finished RPC procedure *OR* SQL batch
                    self.handle_done(done);
                }
            }
        }

        Ok(())
    }

    fn handle_done(&mut self, _: Done) {
        self.pending_done_count -= 1;
    }

    async fn run(&mut self, query: &str, arguments: Option<MssqlArguments>) -> Result<(), Error> {
        self.wait_until_ready().await?;
        self.pending_done_count += 1;

        if let Some(mut arguments) = arguments {
            let proc = Either::Right(Procedure::ExecuteSql);
            let mut proc_args = MssqlArguments::default();

            // SQL
            proc_args.add_unnamed(query);

            if !arguments.data.is_empty() {
                // Declarations
                //  NAME TYPE, NAME TYPE, ...
                proc_args.add_unnamed(&*arguments.declarations);

                // Add the list of SQL parameters _after_ our RPC parameters
                proc_args.append(&mut arguments);
            }

            self.stream.write_packet(
                PacketType::Rpc,
                RpcRequest {
                    transaction_descriptor: self.stream.transaction_descriptor,
                    arguments: &proc_args,
                    procedure: proc,
                    options: OptionFlags::empty(),
                },
            );
        } else {
            self.stream.write_packet(
                PacketType::SqlBatch,
                SqlBatch {
                    transaction_descriptor: self.stream.transaction_descriptor,
                    sql: query,
                },
            );
        }

        self.stream.flush().await?;

        Ok(())
    }
}

impl<'c> Executor<'c> for &'c mut MssqlConnection {
    type Database = Mssql;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<u64, MssqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream2! {
            self.run(s, arguments).await?;

            loop {
                let message = self.stream.recv_message().await?;

                match message {
                    Message::Row(row) => {
                        r#yield!(Either::Right(MssqlRow { row }));
                    }

                    Message::Done(done) | Message::DoneProc(done) => {
                        if done.status.contains(Status::DONE_COUNT) {
                            r#yield!(Either::Left(done.affected_rows));
                        }

                        if !done.status.contains(Status::DONE_MORE) {
                            self.handle_done(done);
                            break;
                        }
                    }

                    Message::DoneInProc(done) => {
                        if done.status.contains(Status::DONE_COUNT) {
                            r#yield!(Either::Left(done.affected_rows));
                        }
                    }

                    _ => {}
                }
            }

            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<MssqlRow>, Error>>
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

    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();

        // [sp_prepare] will emit the column meta data
        // small issue is that we need to declare all the used placeholders with a "fallback" type
        // we currently use regex to collect them; false positives are *okay* but false
        // negatives would break the query
        let proc = Either::Right(Procedure::Prepare);

        // NOTE: this does not support unicode identifiers; as we don't even support
        //       named parameters (yet) this is probably fine, for now

        static PARAMS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"@p[[:alnum:]]+").unwrap());

        let mut params = String::new();
        let mut num_params = 0;

        for m in PARAMS_RE.captures_iter(s) {
            if !params.is_empty() {
                params.push_str(",");
            }

            params.push_str(&m[0]);

            // NOTE: this means that a query! of `SELECT @p1` will have the macros believe
            //       it will return nvarchar(1); this is a greater issue with `query!` that we
            //       we need to circle back to. This doesn't happen much in practice however.
            params.push_str(" nvarchar(1)");

            num_params += 1;
        }

        let params = if params.is_empty() {
            None
        } else {
            Some(&*params)
        };

        let mut args = MssqlArguments::default();

        args.declare("", 0_i32);
        args.add_unnamed(params);
        args.add_unnamed(s);
        args.add_unnamed(0x0001_i32); // 1 = SEND_METADATA

        self.stream.write_packet(
            PacketType::Rpc,
            RpcRequest {
                transaction_descriptor: self.stream.transaction_descriptor,
                arguments: &args,
                procedure: proc,
                options: OptionFlags::empty(),
            },
        );

        Box::pin(async move {
            self.stream.flush().await?;

            loop {
                match self.stream.recv_message().await? {
                    Message::DoneProc(done) | Message::Done(done) => {
                        if !done.status.contains(Status::DONE_MORE) {
                            // done with prepare
                            break;
                        }
                    }

                    _ => {}
                }
            }

            let mut columns = Vec::with_capacity(self.stream.columns.len());

            for col in &self.stream.columns {
                columns.push(Column {
                    name: col.col_name.clone(),
                    type_info: Some(MssqlTypeInfo(col.type_info.clone())),
                    not_null: Some(!col.flags.contains(Flags::NULLABLE)),
                });
            }

            Ok(Describe {
                params: vec![None; num_params],
                columns,
            })
        })
    }
}
