use super::Connection;
use futures::{io::AsyncWrite, ready, Stream};
use sqlx_postgres_protocol::{self as proto, Encode, Parse};
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

// NOTE: This is a rough draft of the implementation

#[inline]
pub fn execute<'a>(connection: &'a mut Connection, query: &'a str) -> Execute<'a> {
    Execute {
        connection,
        query,
        state: ExecuteState::Parse,
        rows: 0,
    }
}

pub struct Execute<'a> {
    connection: &'a mut Connection,
    query: &'a str,
    state: ExecuteState,
    rows: u64,
}

#[derive(Debug)]
enum ExecuteState {
    Parse,
    Bind,
    Execute,
    Sync,
    SendingParse,
    SendingBind,
    SendingExecute,
    SendingSync,
    Flush,
    WaitForComplete,
}

impl<'a> Execute<'a> {
    #[inline]
    pub fn bind(self, value: &'a [u8]) -> Bind<'a, &'a [u8]> {
        Bind { ex: self, value }
    }
}

fn poll_write_all<W: AsyncWrite + Unpin>(
    mut writer: W,
    buf: &mut Vec<u8>,
    cx: &mut Context,
) -> Poll<io::Result<()>> {
    // Derived from https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.16/src/futures_util/io/write_all.rs.html#26
    while !buf.is_empty() {
        let n = ready!(Pin::new(&mut writer).poll_write(cx, &*buf))?;

        buf.truncate(buf.len() - n);

        if n == 0 {
            return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
        }
    }

    Poll::Ready(Ok(()))
}

fn poll_execute<T: ToSql>(
    cx: &mut Context,
    conn: &mut Connection,
    state: &mut ExecuteState,
    query: &str,
    values: &T,
    out: &mut u64,
) -> Poll<io::Result<u64>> {
    loop {
        *state = match state {
            ExecuteState::Parse => {
                conn.wbuf.clear();

                let stmt = format!(
                    "__sqlx#{}",
                    conn.statement_index.fetch_add(1, Ordering::SeqCst)
                );
                Parse::new(&stmt, query, &[])
                    .encode(&mut conn.wbuf)
                    .unwrap();

                ExecuteState::SendingParse
            }

            ExecuteState::SendingParse => {
                ready!(poll_write_all(&mut conn.stream.inner, &mut conn.wbuf, cx))?;

                ExecuteState::Bind
            }

            ExecuteState::Bind => {
                conn.wbuf.clear();

                // FIXME: Think of a better way to build up a BIND message. Think on how to
                //        avoid allocation here.

                let mut values_buf = Vec::new();
                values_buf.extend_from_slice(&values.count().to_be_bytes());
                values.to_sql(&mut values_buf);

                // FIXME: We need to cache the statement name around
                let stmt = format!("__sqlx#{}", conn.statement_index.load(Ordering::SeqCst) - 1);

                proto::Bind::new(&stmt, &stmt, &[], &values_buf, &[])
                    .encode(&mut conn.wbuf)
                    .unwrap();

                ExecuteState::SendingBind
            }

            ExecuteState::SendingBind => {
                ready!(poll_write_all(&mut conn.stream.inner, &mut conn.wbuf, cx))?;

                ExecuteState::Execute
            }

            ExecuteState::Execute => {
                conn.wbuf.clear();

                // FIXME: We need to cache the statement name around
                let stmt = format!("__sqlx#{}", conn.statement_index.load(Ordering::SeqCst) - 1);

                proto::Execute::new(&stmt, 0)
                    .encode(&mut conn.wbuf)
                    .unwrap();

                ExecuteState::SendingExecute
            }

            ExecuteState::SendingExecute => {
                ready!(poll_write_all(&mut conn.stream.inner, &mut conn.wbuf, cx))?;

                ExecuteState::Sync
            }

            ExecuteState::Sync => {
                conn.wbuf.clear();
                proto::Sync.encode(&mut conn.wbuf).unwrap();

                ExecuteState::SendingSync
            }

            ExecuteState::SendingSync => {
                ready!(poll_write_all(&mut conn.stream.inner, &mut conn.wbuf, cx))?;

                ExecuteState::Flush
            }

            ExecuteState::Flush => {
                ready!(Pin::new(&mut conn.stream.inner).poll_flush(cx))?;

                ExecuteState::WaitForComplete
            }

            ExecuteState::WaitForComplete => {
                while let Some(message) = ready!(Pin::new(&mut conn.stream).poll_next(cx)) {
                    match message? {
                        proto::Message::BindComplete | proto::Message::ParseComplete => {
                            // Indicates successful completion of a phase
                        }

                        proto::Message::DataRow(_) => {
                            // This is EXECUTE so we are ignoring any potential results
                        }

                        proto::Message::CommandComplete(body) => {
                            *out = body.rows();
                        }

                        proto::Message::ReadyForQuery(_) => {
                            // Successful completion of the whole cycle
                            return Poll::Ready(Ok(*out));
                        }

                        message => {
                            unimplemented!("received {:?} unimplemented message", message);
                        }
                    }
                }

                // FIXME: This is technically reachable if the pg conn is dropped?
                unreachable!()
            }
        }
    }
}

impl<'a> Future for Execute<'a> {
    type Output = io::Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let self_ = self.get_mut();
        poll_execute(
            cx,
            &mut *self_.connection,
            &mut self_.state,
            &self_.query,
            &(),
            &mut self_.rows,
        )
    }
}

// TODO: This should be cleaned up and moved to core; probably needs to be generic over back-end
// TODO: I'm using some trait recursion here.. this should probably not be exposed in core
pub trait ToSql {
    /// Converts the value of `self` into the appropriate format, appending it to `out`.
    fn to_sql(&self, out: &mut Vec<u8>);

    // Count the number of value parameters recursively encoded.
    fn count(&self) -> i16;
}

impl<'a> ToSql for () {
    #[inline]
    fn to_sql(&self, _out: &mut Vec<u8>) {
        // Do nothing
    }

    #[inline]
    fn count(&self) -> i16 {
        0
    }
}

impl<'a> ToSql for &'a [u8] {
    #[inline]
    fn to_sql(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.len() as i32).to_be_bytes());
        out.extend_from_slice(self);
    }

    #[inline]
    fn count(&self) -> i16 {
        1
    }
}

impl<'a, T: ToSql + 'a, U: ToSql + 'a> ToSql for (T, U) {
    #[inline]
    fn to_sql(&self, out: &mut Vec<u8>) {
        self.0.to_sql(out);
        self.1.to_sql(out);
    }

    #[inline]
    fn count(&self) -> i16 {
        self.0.count() + self.1.count()
    }
}

pub struct Bind<'a, T: ToSql + 'a> {
    ex: Execute<'a>,
    value: T,
}

impl<'a, T: ToSql + 'a> Bind<'a, T> {
    #[inline]
    pub fn bind(self, value: &'a [u8]) -> Bind<'a, (T, &'a [u8])> {
        Bind {
            ex: self.ex,
            value: (self.value, value),
        }
    }
}

impl<'a, T: Unpin + ToSql + 'a> Future for Bind<'a, T> {
    type Output = io::Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let self_ = self.get_mut();
        poll_execute(
            cx,
            &mut *self_.ex.connection,
            &mut self_.ex.state,
            &self_.ex.query,
            &self_.value,
            &mut self_.ex.rows,
        )
    }
}
