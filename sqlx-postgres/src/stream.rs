use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use bytes::Buf;
use sqlx_core::io::{BufStream, Serialize, Stream};
use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Result, Runtime};

use crate::protocol::backend::{BackendMessage, BackendMessageType};
use crate::protocol::frontend::Terminate;

/// Reads and writes messages to and from the PostgreSQL database server.
///
/// The logic for serializing data structures into the messages is found
/// mostly in `protocol/`.
///
/// The first byte of a message identifies the message type, and the next
/// four bytes specify the length of the rest of the message (this length
/// count includes itself, but not the message-type byte). The remaining
/// contents of the message are determined by the message type. For
/// historical reasons, the very first message sent by the client (
/// the startup message) has no initial message-type byte.
///
/// <https://dev.postgres.com/doc/internals/en/postgres-packet.html>
///
#[allow(clippy::module_name_repetitions)]
pub(crate) struct PgStream<Rt: Runtime> {
    stream: BufStream<Rt, NetStream<Rt>>,
}

impl<Rt: Runtime> PgStream<Rt> {
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self { stream: BufStream::with_capacity(stream, 4096, 1024) }
    }

    // all communication is through a stream of messages
    pub(crate) fn write_message<'ser, T>(&'ser mut self, message: &T) -> Result<()>
    where
        T: Serialize<'ser> + Debug,
    {
        log::trace!("write > {:?}", message);

        message.serialize(self.stream.buffer())?;

        Ok(())
    }

    // reads and consumes a message from the stream buffer
    // assumes there is a message on the stream
    fn read_message(&mut self, size: usize) -> Result<Option<BackendMessage>> {
        // the first byte is the message type
        let ty = self.stream.get(0, 1).get_u8();
        let ty: BackendMessageType = ty.try_into()?;

        // the next 4 bytes was the length of the message
        self.stream.consume(5);

        // and now take the message contents
        let contents = self.stream.take(size);

        if contents.len() != size {
            // TODO: return a database error
            // BUG: something is very wrong somewhere if this branch is executed
            //      either in the SQLx Postgres driver or in the Postgres server
            unimplemented!(
                "Received {} bytes for packet but expecting {} bytes",
                contents.len(),
                size
            );
        }

        match ty {
            BackendMessageType::ErrorResponse => {
                // TODO: return a proper error
                unimplemented!("error response");
            }

            BackendMessageType::NotificationResponse => {
                // TODO: handle these similar to master
                Ok(None)
            }

            BackendMessageType::NoticeResponse => {
                // TODO: log the incoming message
                Ok(None)
            }

            BackendMessageType::ParameterStatus => {
                // TODO: pull out and remember server version
                Ok(None)
            }

            _ => Ok(Some(BackendMessage { contents, ty })),
        }
    }
}

macro_rules! impl_read_message {
    ($(@$blocking:ident)? $self:ident) => {{
        Ok(loop {
            // reads at least 5 bytes from the IO stream into the read buffer
            impl_read_message!($(@$blocking)? @stream $self, 0, 5);

            // bytes 1..4 will be the length of the message
            let size = ($self.stream.get(1, 4).get_u32() - 4) as usize;

            // read <size> bytes _after_ the header
            impl_read_message!($(@$blocking)? @stream $self, 4, size);

            if let Some(message) = $self.read_message(size)? {
                break message;
            }
        })
    }};

    (@blocking @stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read($offset, $n)?;
    };

    (@stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read_async($offset, $n).await?;
    };
}

impl<Rt: Runtime> PgStream<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn read_message_async(&mut self) -> Result<BackendMessage>
    where
        Rt: sqlx_core::Async,
    {
        impl_read_message!(self)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn read_message_blocking(&mut self) -> Result<BackendMessage>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_read_message!(@blocking self)
    }
}

impl<Rt: Runtime> Deref for PgStream<Rt> {
    type Target = BufStream<Rt, NetStream<Rt>>;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<Rt: Runtime> DerefMut for PgStream<Rt> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

macro_rules! read_message {
    (@blocking $stream:expr) => {
        $stream.read_message_blocking()?
    };

    ($stream:expr) => {
        $stream.read_message_async().await?
    };
}

impl<Rt: Runtime> PgStream<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn close_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        self.write_message(&Terminate)?;
        self.flush_async().await?;
        self.shutdown_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn close_blocking(&mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        self.write_message(&Terminate)?;
        self.flush()?;
        self.shutdown()?;

        Ok(())
    }
}
