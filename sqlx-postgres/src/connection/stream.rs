//! Reads and writes packets to and from the PostgreSQL database server.
//!
//! The logic for serializing data structures into the packets is found
//! mostly in `protocol/`.
//!
//! Packets in PostgreSQL are prefixed by 4 bytes.
//! 3 for length (in LE) and a sequence id.
//!
//! Packets may only be as large as the communicated size in the initial
//! `HandshakeResponse`. By default, SQLx configures its chunk size to 16M. Sending
//! a larger payload is simply sending completely "full" packets, one after the
//! other, with an increasing sequence id.
//!
//! In other words, when we sent data, we:
//!
//! -   Split the data into "packets" of size `2 ** 24 - 1` bytes.
//!
//! -   Prepend each packet with a **packet header**, consisting of the length of that packet,
//!     and the sequence number.
//!
//! https://dev.postgres.com/doc/internals/en/postgres-packet.html
//!
use std::convert::TryFrom;
use std::fmt::Debug;

use bytes::{Buf, Bytes};
use log::Level;
use sqlx_core::io::{Deserialize, Serialize};
use sqlx_core::{Result, Runtime};

use crate::protocol::{Message, MessageType, Notice, PgSeverity};
use crate::PostgresConnection;

impl<Rt> PostgresConnection<Rt>
where
    Rt: Runtime,
{
    pub(super) fn write_packet<'ser, T>(&'ser mut self, packet: &T) -> Result<()>
    where
        T: Serialize<'ser, ()> + Debug,
    {
        log::trace!("write > {:?}", packet);

        let buf = self.stream.buffer();
        packet.serialize_with(buf, ())?;

        Ok(())
    }

    pub(crate) fn recv_message(&mut self) -> Result<Message> {
        // all packets in postgres start with a 5-byte header
        // this header contains the message type and the total length of the message
        let mut header: Bytes = self.stream.take(5);

        let r#type = MessageType::try_from(header.get_u8())?;
        let size = (header.get_u32() - 4) as usize;

        let contents = self.stream.take(size);

        Ok(Message { r#type, contents })
    }

    fn recv_packet<'de, T>(&'de mut self, len: usize) -> Result<T>
    where
        T: Deserialize<'de, ()> + Debug,
    {
        loop {
            let message = self.recv_message()?;

            match message.r#type {
                MessageType::ErrorResponse => {
                    // An error returned from the database server.
                    // return Err(PgDatabaseError(message.decode()?).into());
                    panic!("got error response");
                }

                MessageType::NotificationResponse => {
                    // if let Some(buffer) = &mut self.notifications {
                    //     let notification: Notification = message.decode()?;
                    //     let _ = self.write_packet(notification);

                    //     continue;
                    // }
                    continue;
                }

                MessageType::ParameterStatus => {
                    // informs the frontend about the current (initial)
                    // setting of backend parameters

                    // we currently have no use for that data so we promptly ignore this message
                    continue;
                }

                MessageType::NoticeResponse => {
                    // do we need this to be more configurable?
                    // if you are reading this comment and think so, open an issue

                    let notice: Notice = message.decode()?;

                    let lvl = match notice.severity() {
                        PgSeverity::Fatal | PgSeverity::Panic | PgSeverity::Error => Level::Error,
                        PgSeverity::Warning => Level::Warn,
                        PgSeverity::Notice => Level::Info,
                        PgSeverity::Debug => Level::Debug,
                        PgSeverity::Info => Level::Trace,
                        PgSeverity::Log => Level::Trace,
                    };

                    if lvl <= log::STATIC_MAX_LEVEL && lvl <= log::max_level() {
                        log::logger().log(
                            &log::Record::builder()
                                .args(format_args!("{}", notice.message()))
                                .level(lvl)
                                .module_path_static(Some("sqlx::postgres::notice"))
                                .file_static(Some(file!()))
                                .line(Some(line!()))
                                .build(),
                        );
                    }

                    continue;
                }

                _ => {}
            }

            return T::deserialize_with(message.contents, ());
        }
    }
}

macro_rules! read_packet {
    ($(@$blocking:ident)? $self:ident) => {{
        // reads at least 4 bytes from the IO stream into the read buffer
        read_packet!($(@$blocking)? @stream $self, 0, 4);

        // the first 3 bytes will be the payload length of the packet (in LE)
        // ALLOW: the max this len will be is 16M
        #[allow(clippy::cast_possible_truncation)]
        let payload_len: usize = $self.stream.get(0, 3).get_uint_le(3) as usize;

        // read <payload_len> bytes _after_ the 4 byte packet header
        // note that we have not yet told the stream we are done with any of
        // these bytes yet. if this next read invocation were to never return (eg., the
        // outer future was dropped), then the next time read_packet_async was called
        // it will re-read the parsed-above packet header. Note that we have NOT
        // mutated `self` _yet_. This is important.
        read_packet!($(@$blocking)? @stream $self, 4, payload_len);

        $self.recv_packet(payload_len)
    }};

    (@blocking @stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read($offset, $n)?;
    };

    (@stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read_async($offset, $n).await?;
    };
}

impl<Rt> PostgresConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]

    pub(super) async fn read_packet_async<'de, T>(&'de mut self) -> Result<T>
    where
        T: Deserialize<'de, ()> + Debug,
        Rt: sqlx_core::Async,
    {
        read_packet!(self)
    }

    #[cfg(feature = "blocking")]

    pub(super) fn read_packet<'de, T>(&'de mut self) -> Result<T>
    where
        T: Deserialize<'de, ()> + Debug,
        Rt: sqlx_core::blocking::Runtime,
    {
        read_packet!(@blocking self)
    }
}
