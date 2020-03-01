use std::convert::TryInto;
use std::net::Shutdown;

use byteorder::NetworkEndian;

use crate::io::{Buf, BufStream, MaybeTlsStream};
use crate::postgres::protocol::{Encode, Message, Response, Severity};
use crate::postgres::PgError;
use crate::url::Url;

pub struct PgStream {
    stream: BufStream<MaybeTlsStream>,

    // Most recently received message
    // Is referenced by our buffered stream
    // Is initialized to ReadyForQuery/0 at the start
    message: (Message, u32),
}

impl PgStream {
    pub(super) async fn new(url: &Url) -> crate::Result<Self> {
        let stream = MaybeTlsStream::connect(&url, 5432).await?;

        Ok(Self {
            stream: BufStream::new(stream),
            message: (Message::ReadyForQuery, 0),
        })
    }

    pub(super) fn shutdown(&self) -> crate::Result<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    #[inline]
    pub(super) fn write<M>(&mut self, message: M)
    where
        M: Encode,
    {
        message.encode(self.stream.buffer_mut());
    }

    #[inline]
    pub(super) async fn flush(&mut self) -> crate::Result<()> {
        Ok(self.stream.flush().await?)
    }

    pub(super) async fn read(&mut self) -> crate::Result<Message> {
        loop {
            // https://www.postgresql.org/docs/12/protocol-overview.html#PROTOCOL-MESSAGE-CONCEPTS

            // All communication is through a stream of messages. The first byte of a message
            // identifies the message type, and the next four bytes specify the length of the rest of
            // the message (this length count includes itself, but not the message-type byte).

            if self.message.1 > 0 {
                // If there is any data in our read buffer we need to make sure we flush that
                // so reading will return the *next* message
                self.stream.consume(self.message.1 as usize);
            }

            let mut header = self.stream.peek(4 + 1).await?;

            let type_ = header.get_u8()?.try_into()?;
            let length = header.get_u32::<NetworkEndian>()? - 4;

            self.message = (type_, length);
            self.stream.consume(4 + 1);

            // Wait until there is enough data in the stream. We then return without actually
            // inspecting the data. This is then looked at later through the [buffer] function
            let _ = self.stream.peek(length as usize).await?;

            match type_ {
                Message::ErrorResponse | Message::NoticeResponse => {
                    let response = Response::read(self.stream.buffer())?;
                    match response.severity {
                        Severity::Error | Severity::Panic | Severity::Fatal => {
                            // This is an error, bubble up as one immediately
                            return Err(crate::Error::Database(Box::new(PgError(response))));
                        }

                        _ => {}
                    }

                    // TODO: Provide some way of receiving these non-critical
                    //       notices from postgres
                    continue;
                }

                _ => {
                    return Ok(type_);
                }
            }
        }
    }

    /// Returns a reference to the internally buffered message.
    ///
    /// This is the body of the message identified by the most recent call
    /// to `read`.
    #[inline]
    pub(super) fn buffer(&self) -> &[u8] {
        &self.stream.buffer()[..(self.message.1 as usize)]
    }
}
