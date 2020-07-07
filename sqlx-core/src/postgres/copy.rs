use crate::error::{Error, Result};
use crate::pool::{Pool, PoolConnection};
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
    pub async fn copy_in_raw(&mut self, statement: &str) -> Result<PgCopyIn<&mut Self>> {
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
    pub async fn copy_in_raw(
        &mut self,
        statement: &str,
    ) -> Result<PgCopyIn<PoolConnection<Postgres>>> {
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

    /// Send a chunk of `COPY` data.
    ///
    /// If you're copying data from an `AsyncRead`, maybe consider [Self::copy_from] instead.
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
    /// You must still call either [Self::finish] or [Self::fail] to complete the process.
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
    /// The server is expected to respond with an error, so that is returned along with
    /// the connection. Included in the error should be the given message.
    ///
    /// `Err` is only returned for an _unexpected_ error.
    pub async fn fail(&mut self, msg: impl Into<String>) -> Result<(C, PgDatabaseError)> {
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
            Err(Error::Database(db)) => Ok((conn, *db.downcast::<PgDatabaseError>())),
            Err(e) => Err(e),
        }
    }

    /// Signal that the `COPY` process is complete.
    ///
    /// The connection is returned along with the number of rows affected.
    pub async fn finish(&mut self) -> Result<(C, u64)> {
        let mut conn = self
            .conn
            .take()
            .expect("CopyWriter::finish: conn taken illegally");

        conn.stream.send(CopyDone).await?;
        let cc: CommandComplete = conn
            .stream
            .recv_expect(MessageFormat::CommandComplete)
            .await?;

        Ok((conn, cc.rows_affected()))
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
