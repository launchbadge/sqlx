//! Support for `LOAD DATA LOCAL INFILE` statements
//!
//! This MySQL feature allows the client to send a local file to the server, which is then
//! loaded into a table. This should be faster than sending the data row-by-row.
//!
//! # Example
//! ```rust,no_run
//! use sqlx::mysql::infile::MySqlPoolInfileExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), sqlx::Error> {
//!     let pool = sqlx::mysql::MySqlPool::connect("mysql://root:password@localhost:3306/sqlx").await?;
//!
//!     let res = {
//!         let mut stream = pool
//!             .load_local_infile("LOAD DATA LOCAL INFILE 'dummy' INTO TABLE testje")
//!             .await?;
//!         stream.send(b"1\n2\n3\n4\n5\n6\n7\n8\n9\n10").await?;
//!         stream.finish().await?
//!     };
//!     println!("{}", res); // 10
//!
//!     Ok(())
//! }
//! ```

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::Poll,
};

use crate::error::Error;
use crate::protocol::response::LocalInfilePacket;
use crate::protocol::text::Query;
use crate::sqlx_core::net::Socket;
use futures_core::{future::BoxFuture, ready};
use futures_io::AsyncWrite;
use sqlx_core::pool::{Pool, PoolConnection};

use crate::{MySql, MySqlConnection};

use super::MySqlStream;

/// Extension of the MySql Pool with support for `LOAD DATA LOCAL INFILE` statements.
pub trait MySqlPoolInfileExt {
    /// Execute the query using the given handler.
    ///
    /// See the [infile](`crate::infile`) module documentation for an example.
    fn load_local_infile<'a>(
        &'a self,
        statement: &'a str,
    ) -> BoxFuture<'a, Result<MySqlLocalInfile<PoolConnection<MySql>>, Error>>;
}

impl MySqlPoolInfileExt for Pool<MySql> {
    fn load_local_infile<'a>(
        &'a self,
        statement: &'a str,
    ) -> BoxFuture<'a, Result<MySqlLocalInfile<PoolConnection<MySql>>, Error>> {
        Box::pin(async { MySqlLocalInfile::begin(self.acquire().await?, statement).await })
    }
}

const MAX_MYSQL_PACKET_SIZE: usize = (1 << 24) - 2;

impl MySqlConnection {
    /// Execute the query, returning a stream that allows you to send data to the server using `LOAD DATA LOCAL INFILE`.
    ///
    /// See the [infile](`crate::infile`) module documentation for an example, and the [`MySqlLocalInfile`] documentation for implemented methods.
    pub async fn load_local_infile(
        &mut self,
        statement: &str,
    ) -> Result<MySqlLocalInfile<&mut Self>, Error> {
        MySqlLocalInfile::begin(self, statement).await
    }
}

/// A stream that allows you to send data to the server using `LOAD DATA LOCAL INFILE`.
///
/// Use this via the [`MySqlPoolInfileExt::load_data_infile`] or [`MySqlConnection::load_local_infile`] methods.
pub struct MySqlLocalInfile<C: DerefMut<Target = MySqlConnection>> {
    conn: C,
    filename: Vec<u8>,
    buf: Vec<u8>,
}

impl<C: DerefMut<Target = MySqlConnection>> MySqlLocalInfile<C> {
    async fn begin(mut conn: C, statement: &str) -> Result<Self, Error> {
        conn.stream.wait_until_ready().await?;
        conn.stream.send_packet(Query(statement)).await?;

        let packet = conn.stream.recv_packet().await?;
        let packet: LocalInfilePacket = packet.decode()?;
        let filename = packet.filename;

        let mut buf = Vec::with_capacity(MAX_MYSQL_PACKET_SIZE);
        buf.extend_from_slice(&[0; 4]);

        Ok(Self {
            conn,
            filename,
            buf,
        })
    }

    /// Get a writer that implements the [`AsyncWrite`] trait from futures::io
    ///
    /// You probably want to buffer writes to this writer, as any write results in a send
    /// of a packet to MySql.
    ///
    /// ### Note: Completion Step Required
    /// You must still call [finish()](Self::finish) to complete the process.
    /// Closing the writer is not enough.
    pub fn get_writer<'a>(&'a mut self) -> InfileWriter<'a> {
        let sequence_id = self.conn.stream.sequence_id;
        InfileWriter::new(&mut self.conn.stream)
    }

    /// Get the filename that MySql requested from the LOCAL INFILE
    pub fn get_filename(&self) -> &[u8] {
        &self.filename
    }

    /// Send data to the database.
    ///
    /// The data is buffered and send to the server in packets of at most 16MB. The data is automatically flushed when the buffer is full.
    ///
    /// ### Note: Completion Step Required
    /// You must still call [finish()](Self::finish) to complete the process.
    pub async fn send(&mut self, source: impl Deref<Target = [u8]>) -> Result<(), Error> {
        let mut right = source.deref();
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

    /// Flush the stream.
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
        self.buf[3] = self.conn.stream.sequence_id;
        self.conn
            .stream
            .socket
            .socket_mut()
            .write(&self.buf[..len + 4])
            .await?;
        self.buf.drain(..len + 4);
        self.conn.stream.sequence_id = self.conn.stream.sequence_id.wrapping_add(1);
        Ok(())
    }

    /// Finish sending the LOCAL INFILE data to the server.
    ///
    /// This must always be called after you're done writing the data.
    /// Returns the number of rows that were inserted.
    pub async fn finish(mut self) -> Result<u64, Error> {
        self.flush().await?;
        self.conn.stream.send_empty_response().await?;
        let packet = self.conn.stream.recv_packet().await?;
        let packet = packet.ok()?;
        Ok(packet.affected_rows)
    }
}

/// A writer that writes to a [`MySqlLocalInfile`] stream.
pub struct InfileWriter<'a> {
    stream: &'a mut MySqlStream,
    send: Option<SendPacket>,
}

impl<'a> InfileWriter<'a> {
    fn new(stream: &'a mut MySqlStream) -> Self {
        Self { stream, send: None }
    }
}

impl<'a> AsyncWrite for InfileWriter<'a> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<futures_io::Result<usize>> {
        let send = self.send.take();

        let mut send = match send {
            Some(send) => send,
            None => {
                let send = SendPacket::new(buf, self.stream.sequence_id);
                self.stream.sequence_id = self.stream.sequence_id.wrapping_add(1);
                send
            }
        };
        Pin::new(&mut send)
            .poll_send(cx, self.stream.socket_mut())
            .map(|x| x.map(|written| written - 4))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<futures_io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<futures_io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

struct SendPacket {
    buf: Vec<u8>,
}

impl SendPacket {
    fn new(data: &[u8], sequence_id: u8) -> Self {
        let mut buf = Vec::with_capacity(data.len() + 4);
        buf.extend_from_slice(&(data.len() as u32).to_le_bytes()[..3]);
        buf.push(sequence_id);
        buf.extend_from_slice(data);
        Self { buf }
    }

    fn poll_send(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        socket: &mut impl Socket,
    ) -> std::task::Poll<futures_io::Result<usize>> {
        let this = &mut *self;

        while !this.buf.is_empty() {
            match socket.try_write(&mut this.buf) {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    ready!(socket.poll_write_ready(cx))?;
                }
                ready => {
                    if let Ok(written) = ready {
                        this.buf.drain(..written);
                    }
                    return Poll::Ready(ready);
                }
            }
        }

        Poll::Ready(Ok(0))
    }
}
