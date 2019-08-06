use super::prepare::Prepare;
use crate::{
    postgres::protocol::{self, DataRow, Message},
    row::{FromRow, Row},
};
use futures::{stream, Stream, TryStreamExt};
use std::io;

impl<'a, 'b> Prepare<'a, 'b> {
    #[inline]
    pub fn select<Record: 'a, T: 'static>(
        self,
    ) -> impl Stream<Item = Result<T, io::Error>> + 'a + Unpin
    where
        T: FromRow<Record>,
    {
        self.select_raw().map_ok(T::from_row)
    }

    // TODO: Better name?
    // TODO: Should this be public?
    fn select_raw(self) -> impl Stream<Item = Result<Row, io::Error>> + 'a + Unpin {
        // FIXME: Manually implement Stream on a new type to avoid the unfold adapter
        stream::unfold(self.finish(), |conn| {
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
                            break Some((Ok(Row(row)), conn));
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
