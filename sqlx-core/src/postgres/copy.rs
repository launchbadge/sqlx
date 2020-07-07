use crate::error::{Error, Result};
use crate::pool::{MaybePooled, Pool, PoolConnection};
use crate::postgres::connection::{PgConnection, PgStream};
use crate::postgres::message::{
    CommandComplete, CopyData, CopyDone, CopyFail, CopyResponse, MessageFormat, Notice, Query,
};
use crate::postgres::{PgDatabaseError, Postgres};
use bytes::{BufMut, BytesMut};
use sqlx_rt::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::convert::TryFrom;
use std::io::Read;
use std::ops::{Deref, DerefMut};

impl PgConnection {
    /// Issue a `COPY FROM STDIN` statement and transition the connection to streaming data
    /// to Postgres.
    ///
    /// If `statement` is anything other than a `COPY ... FROM STDIN ...` command, an error is
    /// returned.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    pub async fn copy_in_raw<'c>(&'c mut self, statement: &str) -> Result<PgCopyIn<'c>> {
        PgCopyIn::begin(self, statement).await
    }
}

impl Pool<Postgres> {
    /// Issue a `COPY FROM STDIN` statement and begin streaming data to Postgres.
    ///
    /// A single connection will be checked out for the duration.
    ///
    /// If `statement` is anything other than a `COPY ... FROM STDIN ...` command, an error is
    /// returned.
    ///
    /// Command examples and accepted formats for `COPY` data are shown here:
    /// https://www.postgresql.org/docs/current/sql-copy.html
    pub async fn copy_in_raw(&mut self, statement: &str) -> Result<PgCopyIn<'static>> {
        PgCopyIn::begin(self.acquire().await?, statement).await
    }
}

/// A connection in streaming `COPY FROM STDIN` mode.
///
/// Created by [PgConnection::copy_in_raw].
///
/// ### Note
/// [Self::finish] or [Self::fail] *must* be called or the connection will return an error
/// the next time it is used.
#[must_use = "connection will error on next use if .finish() or .fail() is not called"]
pub struct PgCopyIn<'c> {
    conn: MaybePooled<'c, Postgres>,
    response: CopyResponse,
    finished: bool,
}

impl<'c> PgCopyIn<'c> {
    async fn begin(conn: impl Into<MaybePooled<'c, Postgres>>, statement: &str) -> Result<Self> {
        let mut conn = conn.into();

        conn.wait_until_ready().await?;
        conn.stream.send(Query(statement)).await?;

        let response: CopyResponse = conn
            .stream
            .recv_expect(MessageFormat::CopyInResponse)
            .await?;

        Ok(PgCopyIn {
            conn,
            response,
            finished: false,
        })
    }

    /// Send a chunk of `COPY` data.
    ///
    /// If you're copying data from an `AsyncRead`, maybe consider [Self::copy_from] instead.
    pub async fn send(&mut self, data: impl Deref<Target = [u8]>) -> Result<&mut Self> {
        self.conn.stream.send(CopyData(data)).await?;

        Ok(self)
    }

    /// Copy data directly from `source` to the database without requiring an intermediate buffer.
    ///
    /// `source` will be read to the end.
    ///
    /// ### Note
    /// You must still call either [Self::finish] or [Self::fail] to complete the process.
    pub async fn read_from(&mut self, mut source: impl AsyncRead + Unpin) -> Result<&mut Self> {
        // this is a separate guard from WriteAndFlush so we can reuse the buffer without zeroing
        struct BufGuard<'s>(&'s mut Vec<u8>);

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                self.0.clear()
            }
        }

        // flush any existing messages in the buffer and clear it
        self.conn.stream.flush().await?;

        {
            let buf_stream = &mut *self.conn.stream;
            let stream = &mut buf_stream.stream;

            // ensures the buffer isn't left in an inconsistent state
            let mut guard = BufGuard(&mut buf_stream.wbuf);

            let buf: &mut Vec<u8> = &mut guard.0;
            buf.push(b'd'); // CopyData format code
            buf.resize(5, 0); // reserve space for the length

            loop {
                let read = match () {
                    // Tokio lets us read into the buffer without zeroing first
                    #[cfg(any(feature = "runtime-tokio", feature = "runtime-actix"))]
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
    /// The given error message can be used for indicating the reason for the abort in the
    /// database logs.
    ///
    /// The server is expected to respond with an error, but we discard that.
    /// `Err` is only returned for an _unexpected_ error.
    pub async fn abort(mut self, msg: impl Into<String>) -> Result<()> {
        self.finished = true;
        self.conn.stream.send(CopyFail::new(msg)).await?;

        match self.conn.stream.recv().await {
            Ok(msg) => Err(err_protocol!(
                "fail_with: expected ErrorResponse, got: {:?}",
                msg.format
            )),
            // FIXME: introspect the response to be sure we're not discarding a different error
            Err(Error::Database(db)) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Signal that the `COPY` process is complete.
    ///
    /// Returns the number of rows affected.
    pub async fn finish(mut self) -> Result<u64> {
        self.finished = true;
        self.conn.stream.send(CopyDone).await?;
        let cc: CommandComplete = self
            .conn
            .stream
            .recv_expect(MessageFormat::CommandComplete)
            .await?;

        Ok(cc.rows_affected())
    }
}

impl<'c> Drop for PgCopyIn<'c> {
    fn drop(&mut self) {
        if !self.finished {
            self.conn.stream.write(CopyFail::new(
                "PgCopyIn dropped without calling finish() or fail()",
            ));
        }
    }
}
