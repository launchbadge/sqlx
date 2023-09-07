use std::ops::DerefMut;

use crate::executor::Execute;
use crate::{error::Error, MySqlPool};
use either::Either;
use futures_core::future::BoxFuture;
use futures_util::{pin_mut, FutureExt, StreamExt, TryStreamExt};
use sqlx_core::database::Database;
use sqlx_core::net::Socket;

use crate::{MySql, MySqlConnection};

use super::MySqlStream;

pub trait MySqlExecutorInfileExt<'c> {
    fn local_infile_statement<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
        infile_handler: LocalInfileHandler,
    ) -> BoxFuture<'e, Result<<MySql as Database>::QueryResult, Error>>
    where
        'c: 'e,
        E: Execute<'q, MySql>;
}

impl<'c> MySqlExecutorInfileExt<'c> for &'c mut MySqlConnection {
    fn local_infile_statement<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
        infile_handler: LocalInfileHandler,
    ) -> BoxFuture<'e, Result<<MySql as Database>::QueryResult, Error>>
    where
        'c: 'e,
        E: Execute<'q, MySql>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        let persistent = query.persistent();

        Box::pin(try_stream! {
            let s = self.run(sql, arguments, persistent, Some(infile_handler)).await?;
            pin_mut!(s);

            while let Some(v) = s.try_next().await? {
                r#yield!(v);
            }

            Ok(())
        })
        .try_filter_map(|step| async move {
            Ok(match step {
                Either::Left(rows) => Some(rows),
                Either::Right(_) => None,
            })
        })
        .boxed()
        .try_collect()
        .boxed()
    }
}

impl<'c> MySqlExecutorInfileExt<'c> for &'_ MySqlPool {
    fn local_infile_statement<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
        infile_handler: LocalInfileHandler,
    ) -> BoxFuture<'e, Result<<MySql as Database>::QueryResult, Error>>
    where
        'c: 'e,
        E: Execute<'q, MySql>,
    {
        let pool = self.clone();
        Box::pin(async move {
            let mut conn = pool.acquire().await?;
            conn.deref_mut()
                .local_infile_statement(query, infile_handler)
                .await
        })
    }
}

pub struct LocalInfileHandler(
    Box<
        dyn for<'a> FnOnce(&'a [u8], &'a mut InfileDataStream) -> BoxFuture<'a, Result<(), Error>>
            + Send
            + 'static,
    >,
);

impl LocalInfileHandler {
    pub fn new<F>(f: F) -> Self
    where
        F: for<'a> FnOnce(&'a [u8], &'a mut InfileDataStream) -> BoxFuture<'a, Result<(), Error>>
            + Send
            + 'static,
    {
        Self(Box::new(f))
    }

    pub(crate) async fn handle(
        self,
        stream: &mut MySqlStream,
        filename: &[u8],
    ) -> Result<(), Error> {
        let mut infiledata = stream.get_data_stream();
        self.0(filename, &mut infiledata).await?;
        infiledata.flush().await?;
        Ok(())
    }
}

const MAX_MYSQL_PACKET_SIZE: usize = (1 << 24) - 2;

pub struct InfileDataStream<'s> {
    stream: &'s mut MySqlStream,
    buf: Vec<u8>,
}

impl<'s> InfileDataStream<'s> {
    pub(crate) fn new(stream: &'s mut MySqlStream) -> Self {
        let mut buf = Vec::with_capacity(MAX_MYSQL_PACKET_SIZE);
        buf.extend_from_slice(&[0; 4]);
        Self { stream, buf }
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut right = buf;
        while !right.is_empty() {
            let (left, right_) = right.split_at(std::cmp::min(MAX_MYSQL_PACKET_SIZE, right.len()));
            self.buf.extend_from_slice(left);
            if self.buf.len() >= MAX_MYSQL_PACKET_SIZE + 4 {
                assert_eq!(self.buf.len(), MAX_MYSQL_PACKET_SIZE + 4);
                self.drain_packet(MAX_MYSQL_PACKET_SIZE).await?;
                assert!(self.buf.is_empty());
                self.buf.extend_from_slice(&[0; 4]);
            }
            right = right_;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        if self.buf.len() > 4 {
            // Cannot have multiple packets in buffer, as they would have been drained by write() already
            assert!(self.buf.len() <= MAX_MYSQL_PACKET_SIZE + 4);
            self.drain_packet(self.buf.len() - 4).await?;
        }
        Ok(())
    }

    async fn drain_packet(&mut self, len: usize) -> Result<(), Error> {
        self.buf[0..3].copy_from_slice(&(len as u32).to_le_bytes()[..3]);
        self.buf[3] = self.stream.sequence_id;
        self.stream
            .socket
            .socket_mut()
            .write(&self.buf[..len + 4])
            .await?;
        self.buf.drain(..len + 4);
        self.stream.sequence_id = self.stream.sequence_id.wrapping_add(1);
        Ok(())
    }
}
