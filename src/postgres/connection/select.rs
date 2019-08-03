use super::prepare::Prepare;
use crate::postgres::protocol::{self, DataRow, Message};
use futures::{stream, Stream};
use std::io;

impl<'a> Prepare<'a> {
    pub fn select(self) -> impl Stream<Item = Result<DataRow, io::Error>> + 'a + Unpin {
        // protocol::bind::trailer(
        //     &mut self.connection.wbuf,
        //     self.bind_state,
        //     self.bind_values,
        //     &[],
        // );

        // protocol::execute(&mut self.connection.wbuf, "", 0);
        // protocol::sync(&mut self.connection.wbuf);

        // FIXME: Manually implement Stream on a new type to avoid the unfold adapter
        stream::unfold(self.connection, |conn| {
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
        })
    }
}
