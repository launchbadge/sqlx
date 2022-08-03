use crate::error::{Error, Result};
use crate::ext::async_stream::TryAsyncStream;
use crate::pool::{Pool, PoolConnection};
use crate::postgres::connection::PgConnection;
use crate::postgres::message::{
    CommandComplete, CopyData, CopyDone, CopyFail, CopyResponse, MessageFormat, Query,
};
use crate::postgres::Postgres;
use bytes::{BufMut, Bytes};
use futures_core::stream::BoxStream;
use smallvec::alloc::borrow::Cow;
use sqlx_rt::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use std::ops::{Deref, DerefMut};

impl PgConnection {
    /// Issue a `COPY FROM STDIN` statement and transition the connection to streaming data
    /// to Postgres. This is a more efficient way to import data into Postgres as compared to
    /// `INSERT` but requires one of a few specific data formats (text/CSV/binary).
    ///
    /// If `statement` is anything other than a `COPY ... FROM STDIN ...` command, an error is
    /// returned.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    ///
    /// ### Note
    /// [PgCopyIn::finish] or [PgCopyIn::abort] *must* be called when finished or the connection
    /// will return an error the next time it is used.
    pub async fn copy_in_raw(&mut self, statement: &str) -> Result<PgCopyIn<&mut Self>> {
        PgCopyIn::begin(self, statement).await
    }

    /// Issue a `COPY TO STDOUT` statement and transition the connection to streaming data
    /// from Postgres. This is a more efficient way to export data from Postgres but
    /// arrives in chunks of one of a few data formats (text/CSV/binary).
    ///
    /// If `statement` is anything other than a `COPY ... TO STDOUT ...` command,
    /// an error is returned.
    ///
    /// Note that once this process has begun, unless you read the stream to completion,
    /// it can only be canceled in two ways:
    ///
    /// 1. by closing the connection, or:
    /// 2. by using another connection to kill the server process that is sending the data as shown
    /// [in this StackOverflow answer](https://stackoverflow.com/a/35319598).
    ///
    /// If you don't read the stream to completion, the next time the connection is used it will
    /// need to read and discard all the remaining queued data, which could take some time.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    #[allow(clippy::needless_lifetimes)]
    pub async fn copy_out_raw<'c>(
        &'c mut self,
        statement: &str,
    ) -> Result<BoxStream<'c, Result<Bytes>>> {
        pg_begin_copy_out(self, statement).await
    }
}

impl Pool<Postgres> {
    /// Issue a `COPY FROM STDIN` statement and begin streaming data to Postgres.
    /// This is a more efficient way to import data into Postgres as compared to
    /// `INSERT` but requires one of a few specific data formats (text/CSV/binary).
    ///
    /// A single connection will be checked out for the duration.
    ///
    /// If `statement` is anything other than a `COPY ... FROM STDIN ...` command, an error is
    /// returned.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    ///
    /// ### Note
    /// [PgCopyIn::finish] or [PgCopyIn::abort] *must* be called when finished or the connection
    /// will return an error the next time it is used.
    pub async fn copy_in_raw(&self, statement: &str) -> Result<PgCopyIn<PoolConnection<Postgres>>> {
        PgCopyIn::begin(self.acquire().await?, statement).await
    }

    /// Issue a `COPY TO STDOUT` statement and begin streaming data
    /// from Postgres. This is a more efficient way to export data from Postgres but
    /// arrives in chunks of one of a few data formats (text/CSV/binary).
    ///
    /// If `statement` is anything other than a `COPY ... TO STDOUT ...` command,
    /// an error is returned.
    ///
    /// Note that once this process has begun, unless you read the stream to completion,
    /// it can only be canceled in two ways:
    ///
    /// 1. by closing the connection, or:
    /// 2. by using another connection to kill the server process that is sending the data as shown
    /// [in this StackOverflow answer](https://stackoverflow.com/a/35319598).
    ///
    /// If you don't read the stream to completion, the next time the connection is used it will
    /// need to read and discard all the remaining queued data, which could take some time.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    pub async fn copy_out_raw(&self, statement: &str) -> Result<BoxStream<'static, Result<Bytes>>> {
        pg_begin_copy_out(self.acquire().await?, statement).await
    }
}

/// A connection in streaming `COPY FROM STDIN` mode.
///
/// Created by [PgConnection::copy_in_raw] or [Pool::copy_out_raw].
///
/// ### Note
/// [PgCopyIn::finish] or [PgCopyIn::abort] *must* be called when finished or the connection
/// will return an error the next time it is used.
#[must_use = "connection will error on next use if `.finish()` or `.abort()` is not called"]
pub struct PgCopyIn<C: DerefMut<Target = PgConnection>> {
    conn: Option<C>,
    response: CopyResponse,
}

impl<C: DerefMut<Target = PgConnection>> PgCopyIn<C> {
    async fn begin(mut conn: C, statement: &str) -> Result<Self> {
        conn.wait_until_ready().await?;
        conn.stream.send(Query(statement)).await?;

        let response: CopyResponse = conn
            .stream
            .recv_expect(MessageFormat::CopyInResponse)
            .await?;

        Ok(PgCopyIn {
            conn: Some(conn),
            response,
        })
    }

    /// Returns `true` if Postgres is expecting data in text or CSV format.
    pub fn is_textual(&self) -> bool {
        self.response.format == 0
    }

    /// Returns the number of columns expected in the input.
    pub fn num_columns(&self) -> usize {
        assert_eq!(
            self.response.num_columns as usize,
            self.response.format_codes.len(),
            "num_columns does not match format_codes.len()"
        );
        self.response.format_codes.len()
    }

    /// Check if a column is expecting data in text format (`true`) or binary format (`false`).
    ///
    /// ### Panics
    /// If `column` is out of range according to [`.num_columns()`][Self::num_columns].
    pub fn column_is_textual(&self, column: usize) -> bool {
        self.response.format_codes[column] == 0
    }

    /// Send a chunk of `COPY` data.
    ///
    /// If you're copying data from an `AsyncRead`, maybe consider [Self::read_from] instead.
    pub async fn send(&mut self, data: impl Deref<Target = [u8]>) -> Result<&mut Self> {
        self.conn
            .as_deref_mut()
            .expect("send_data: conn taken")
            .stream
            .send(CopyData(data))
            .await?;

        Ok(self)
    }

    /// Copy data directly from `source` to the database without requiring an intermediate buffer.
    ///
    /// `source` will be read to the end.
    ///
    /// ### Note
    /// You must still call either [Self::finish] or [Self::abort] to complete the process.
    pub async fn read_from(&mut self, mut source: impl AsyncRead + Unpin) -> Result<&mut Self> {
        // this is a separate guard from WriteAndFlush so we can reuse the buffer without zeroing
        struct BufGuard<'s>(&'s mut Vec<u8>);

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                self.0.clear()
            }
        }

        let conn: &mut PgConnection = self.conn.as_deref_mut().expect("copy_from: conn taken");

        // flush any existing messages in the buffer and clear it
        conn.stream.flush().await?;

        {
            let buf_stream = &mut *conn.stream;
            let stream = &mut buf_stream.stream;

            // ensures the buffer isn't left in an inconsistent state
            let mut guard = BufGuard(&mut buf_stream.wbuf);

            let buf: &mut Vec<u8> = &mut guard.0;
            buf.push(b'd'); // CopyData format code
            buf.resize(5, 0); // reserve space for the length

            loop {
                let read = match () {
                    // Tokio lets us read into the buffer without zeroing first
                    #[cfg(feature = "runtime-tokio")]
                    _ if buf.len() != buf.capacity() => {
                        // in case we have some data in the buffer, which can occur
                        // if the previous write did not fill the buffer
                        buf.truncate(5);
                        source.read_buf(buf).await?
                    }
                    _ => {
                        // should be a no-op unless len != capacity
                        buf.resize(buf.capacity(), 0);
                        source.read(&mut buf[5..]).await?
                    }
                };

                if read == 0 {
                    break;
                }

                let read32 = u32::try_from(read)
                    .map_err(|_| err_protocol!("number of bytes read exceeds 2^32: {}", read))?;

                (&mut buf[1..]).put_u32(read32 + 4);

                stream.write_all(&buf[..read + 5]).await?;
                stream.flush().await?;
            }
        }

        Ok(self)
    }

    /// Signal that the `COPY` process should be aborted and any data received should be discarded.
    ///
    /// The given message can be used for indicating the reason for the abort in the database logs.
    ///
    /// The server is expected to respond with an error, so only _unexpected_ errors are returned.
    pub async fn abort(mut self, msg: impl Into<String>) -> Result<()> {
        let mut conn = self
            .conn
            .take()
            .expect("PgCopyIn::fail_with: conn taken illegally");

        conn.stream.send(CopyFail::new(msg)).await?;

        match conn.stream.recv().await {
            Ok(msg) => Err(err_protocol!(
                "fail_with: expected ErrorResponse, got: {:?}",
                msg.format
            )),
            Err(Error::Database(e)) => {
                match e.code() {
                    Some(Cow::Borrowed("57014")) => {
                        // postgres abort received error code
                        conn.stream
                            .recv_expect(MessageFormat::ReadyForQuery)
                            .await?;
                        Ok(())
                    }
                    _ => Err(Error::Database(e)),
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Signal that the `COPY` process is complete.
    ///
    /// The number of rows affected is returned.
    pub async fn finish(mut self) -> Result<u64> {
        let mut conn = self
            .conn
            .take()
            .expect("CopyWriter::finish: conn taken illegally");

        conn.stream.send(CopyDone).await?;
        let cc: CommandComplete = conn
            .stream
            .recv_expect(MessageFormat::CommandComplete)
            .await?;

        conn.stream
            .recv_expect(MessageFormat::ReadyForQuery)
            .await?;

        Ok(cc.rows_affected())
    }
}

impl<C: DerefMut<Target = PgConnection>> Drop for PgCopyIn<C> {
    fn drop(&mut self) {
        if let Some(mut conn) = self.conn.take() {
            conn.stream.write(CopyFail::new(
                "PgCopyIn dropped without calling finish() or fail()",
            ));
        }
    }
}

async fn pg_begin_copy_out<'c, C: DerefMut<Target = PgConnection> + Send + 'c>(
    mut conn: C,
    statement: &str,
) -> Result<BoxStream<'c, Result<Bytes>>> {
    conn.wait_until_ready().await?;
    conn.stream.send(Query(statement)).await?;

    let _: CopyResponse = conn
        .stream
        .recv_expect(MessageFormat::CopyOutResponse)
        .await?;

    let stream: TryAsyncStream<'c, Bytes> = try_stream! {
        loop {
            let msg = conn.stream.recv().await?;
            match msg.format {
                MessageFormat::CopyData => r#yield!(msg.decode::<CopyData<Bytes>>()?.0),
                MessageFormat::CopyDone => {
                    let _ = msg.decode::<CopyDone>()?;
                    conn.stream.recv_expect(MessageFormat::CommandComplete).await?;
                    conn.stream.recv_expect(MessageFormat::ReadyForQuery).await?;
                    return Ok(())
                },
                _ => return Err(err_protocol!("unexpected message format during copy out: {:?}", msg.format))
            }
        }
    };

    Ok(Box::pin(stream))
}
