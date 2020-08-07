use crate::codec::backend::{MessageFormat, RawMessage};
use crate::PgConnection;
use bytes::{Buf, Bytes};
use sqlx_core::error::Error;

impl PgConnection {
    /// Wait for a specific message from the database server.
    pub(crate) async fn recv_exact(&mut self, format: MessageFormat) -> Result<RawMessage, Error> {
        loop {
            let message = self.recv().await?;

            if message.format == format {
                return Ok(message);
            }
        }
    }

    /// Wait for the next message from the database server.
    /// Handles standard and asynchronous messages.
    pub(crate) async fn recv(&mut self) -> Result<RawMessage, Error> {
        loop {
            let message = self.recv_unchecked().await?;

            match message.format {
                MessageFormat::ErrorResponse => {
                    // an error was returned from the database
                    todo!("errors: {:?}", message.contents)
                }

                MessageFormat::NotificationResponse => {
                    // a notification was received; this connection has had `LISTEN` ran on it
                    todo!("notifications");
                    continue;
                }

                MessageFormat::ParameterStatus => {
                    // informs the frontend about the current
                    // setting of backend parameters

                    // we currently have no use for that data so we ignore this message
                    continue;
                }

                _ => {}
            }

            return Ok(message);
        }
    }

    /// Wait for the next message from the database server.
    pub(crate) async fn recv_unchecked(&mut self) -> Result<RawMessage, Error> {
        let mut header = self.stream.peek(0, 5).await?;
        // if the future for this method is dropped now, we will re-peek the same header

        // the first byte of a message identifies the message type
        let kind = header.get_u8();

        // and the next four bytes specify the length of the rest of the message (
        // this length count includes itself, but not the message-type byte).
        let length = header.get_i32() as usize - 4;

        let contents = self.stream.read(5, length).await?;
        // now the packet is fully consumed from the stream and when this method is called
        // again, it will get the *next* message

        Ok(RawMessage {
            format: MessageFormat::try_from_u8(kind)?,
            contents,
        })
    }
}
