use async_stream::try_stream;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;

use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::mssql::protocol::message::Message;
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::protocol::sql_batch::SqlBatch;
use crate::mssql::{MsSql, MsSqlConnection, MsSqlRow};

impl MsSqlConnection {
    async fn run(&mut self, query: &str) -> Result<(), Error> {
        self.stream
            .write_packet(PacketType::SqlBatch, SqlBatch { sql: query });

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
        // TODO: let arguments = query.take_arguments();

        Box::pin(try_stream! {
            self.run(s).await?;

            loop {
                match self.stream.recv_message().await? {
                    Message::Row(row) => {
                        let v = Either::Right(MsSqlRow { row });
                        yield v;
                    }

                    Message::Done(done) => {
                        let v = Either::Left(done.affected_rows);
                        yield v;

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
