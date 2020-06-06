use async_stream::try_stream;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;

use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::mssql::protocol::done::Done;
use crate::mssql::protocol::message::Message;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::rpc::{OptionFlags, Procedure, RpcRequest};
use crate::mssql::protocol::sql_batch::SqlBatch;
use crate::mssql::{MsSql, MsSqlArguments, MsSqlConnection, MsSqlRow};

impl MsSqlConnection {
    async fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.stream.wbuf.is_empty() {
            self.stream.flush().await?;
        }

        while self.pending_done_count > 0 {
            if let Message::DoneProc(done) | Message::Done(done) =
                self.stream.recv_message().await?
            {
                // finished RPC procedure *OR* SQL batch
                self.handle_done(done);
            }
        }

        Ok(())
    }

    fn handle_done(&mut self, _: Done) {
        self.pending_done_count -= 1;
    }

    async fn run(&mut self, query: &str, arguments: Option<MsSqlArguments>) -> Result<(), Error> {
        self.wait_until_ready().await?;
        self.pending_done_count += 1;

        if let Some(mut arguments) = arguments {
            let proc = Either::Right(Procedure::ExecuteSql);
            let mut proc_args = MsSqlArguments::default();

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
                    arguments: &proc_args,
                    procedure: proc,
                    options: OptionFlags::empty(),
                },
            );
        } else {
            self.stream
                .write_packet(PacketType::SqlBatch, SqlBatch { sql: query });
        }

        self.stream.flush().await?;

        Ok(())
    }
}

impl<'c> Executor<'c> for &'c mut MsSqlConnection {
    type Database = MsSql;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<u64, MsSqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream! {
            self.run(s, arguments).await?;

            loop {
                match self.stream.recv_message().await? {
                    Message::Row(row) => {
                        let v = Either::Right(MsSqlRow { row });
                        yield v;
                    }

                    Message::DoneProc(done) => {
                        self.handle_done(done);
                        break;
                    }

                    Message::DoneInProc(done) => {
                        // finished SQL query *within* procedure
                        let v = Either::Left(done.affected_rows);
                        yield v;
                    }

                    Message::Done(done) => {
                        self.handle_done(done);
                        break;
                    }

                    _ => {}
                }
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<MsSqlRow>, Error>>
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
        unimplemented!()
    }
}
