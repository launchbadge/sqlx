use super::Connection;
use futures::{stream, Stream};
use sqlx_postgres_protocol::{self as proto, DataRow, Execute, Message, Parse, Sync};
use std::io;

pub struct Prepare<'a> {
    connection: &'a mut Connection,
    bind_state: (usize, usize),
    bind_values: usize,
}

#[inline]
pub fn prepare<'a, 'b>(connection: &'a mut Connection, query: &'b str) -> Prepare<'a> {
    // TODO: Use a hash map to cache the parse
    // TODO: Use named statements
    connection.send(Parse::new("", query, &[]));

    let bind_state = proto::bind::header(&mut connection.wbuf, "", "", &[]);

    Prepare {
        connection,
        bind_state,
        bind_values: 0,
    }
}

impl<'a> Prepare<'a> {
    #[inline]
    pub fn bind<'b>(mut self, value: &'b [u8]) -> Self {
        proto::bind::value(&mut self.connection.wbuf, value);
        self.bind_values += 1;
        self
    }

    #[inline]
    pub fn bind_null<'b>(mut self) -> Self {
        proto::bind::value_null(&mut self.connection.wbuf);
        self.bind_values += 1;
        self
    }

    #[inline]
    pub async fn execute(self) -> io::Result<u64> {
        proto::bind::trailer(
            &mut self.connection.wbuf,
            self.bind_state,
            self.bind_values,
            &[],
        );

        self.connection.send(Execute::new("", 0));
        self.connection.send(Sync);
        self.connection.flush().await?;

        let mut rows = 0;

        while let Some(message) = self.connection.receive().await? {
            match message {
                Message::BindComplete | Message::ParseComplete => {
                    // Indicates successful completion of a phase
                }

                Message::DataRow(_) => {
                    // This is EXECUTE so we are ignoring any potential results
                }

                Message::CommandComplete(body) => {
                    rows = body.rows();
                }

                Message::ReadyForQuery(_) => {
                    // Successful completion of the whole cycle
                    return Ok(rows);
                }

                message => {
                    unimplemented!("received {:?} unimplemented message", message);
                }
            }
        }

        // FIXME: This is an end-of-file error. How we should bubble this up here?
        unreachable!()
    }

    #[inline]
    pub async fn get_result(self) -> io::Result<Option<DataRow>> {
        proto::bind::trailer(
            &mut self.connection.wbuf,
            self.bind_state,
            self.bind_values,
            &[],
        );

        self.connection.send(Execute::new("", 0));
        self.connection.send(Sync);
        self.connection.flush().await?;

        let mut row: Option<DataRow> = None;

        while let Some(message) = self.connection.receive().await? {
            match message {
                Message::BindComplete | Message::ParseComplete => {
                    // Indicates successful completion of a phase
                }

                Message::DataRow(data_row) => {
                    // we only care about the first result.
                    if row.is_none() {
                        row = Some(data_row);
                    }
                }

                Message::CommandComplete(_) => {}

                Message::ReadyForQuery(_) => {
                    // Successful completion of the whole cycle
                    return Ok(row);
                }

                message => {
                    unimplemented!("received {:?} unimplemented message", message);
                }
            }
        }

        // FIXME: This is an end-of-file error. How we should bubble this up here?
        unreachable!()
    }

    #[inline]
    pub fn get_results(self) -> impl Stream<Item = Result<DataRow, io::Error>> + 'a + Unpin {
        proto::bind::trailer(
            &mut self.connection.wbuf,
            self.bind_state,
            self.bind_values,
            &[],
        );

        self.connection.send(Execute::new("", 0));
        self.connection.send(Sync);

        stream::unfold(self.connection, Self::unfold_func)
    }

    fn unfold_func(
        conn: &mut Connection,
    ) -> impl std::future::Future<Output = Option<(Result<DataRow, io::Error>, &mut Connection)>>
    {
        Box::pin(async {
            if !conn.wbuf.is_empty() {
                if let Err(e) = conn.flush().await {
                    return Some((Err(e), conn));
                }
            }

            loop {
                let message = match conn.receive().await {
                    Ok(Some(message)) => message,
                    // FIXME: This is an end-of-file error. How we should bubble this up here?
                    Ok(None) => unreachable!(),
                    Err(e) => return Some((Err(e), conn)),
                };

                match message {
                    Message::BindComplete | Message::ParseComplete => {
                        // Indicates successful completion of a phase
                    }

                    Message::DataRow(row) => {
                        break Some((Ok(row), conn));
                    }

                    Message::CommandComplete(_) => {}

                    Message::ReadyForQuery(_) => {
                        // Successful completion of the whole cycle
                        break None;
                    }

                    message => {
                        unimplemented!("received {:?} unimplemented message", message);
                    }
                }
            }
        })
    }
}
