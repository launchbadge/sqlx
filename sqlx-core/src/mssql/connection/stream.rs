use std::ops::{Deref, DerefMut};

use bytes::Bytes;
use sqlx_rt::{TcpStream, TlsStream};

use crate::error::Error;
use crate::io::{BufStream, Encode};
use crate::mssql::protocol::col_meta_data::{ColMetaData, ColumnData};
use crate::mssql::protocol::done::Done;
use crate::mssql::protocol::env_change::EnvChange;
use crate::mssql::protocol::error::Error as ProtocolError;
use crate::mssql::protocol::info::Info;
use crate::mssql::protocol::login_ack::LoginAck;
use crate::mssql::protocol::message::{Message, MessageType};
use crate::mssql::protocol::packet::{PacketHeader, PacketType, Status};
use crate::mssql::protocol::return_status::ReturnStatus;
use crate::mssql::protocol::row::Row;
use crate::mssql::{MsSqlConnectOptions, MsSqlDatabaseError};
use crate::net::MaybeTlsStream;

pub(crate) struct MsSqlStream {
    inner: BufStream<MaybeTlsStream<TcpStream>>,

    // current TabularResult from the server that we are iterating over
    response: Option<(PacketHeader, Bytes)>,

    // most recent column data from ColMetaData
    // we need to store this as its needed when decoding <Row>
    columns: Vec<ColumnData>,
}

impl MsSqlStream {
    pub(super) async fn connect(options: &MsSqlConnectOptions) -> Result<Self, Error> {
        let inner = BufStream::new(MaybeTlsStream::Raw(
            TcpStream::connect((&*options.host, options.port)).await?,
        ));

        Ok(Self {
            inner,
            columns: Vec::new(),
            response: None,
        })
    }

    // writes the packet out to the write buffer
    // will (eventually) handle packet chunking
    pub(super) fn write_packet<'en, T: Encode<'en>>(&mut self, ty: PacketType, payload: T) {
        // TODO: Support packet chunking for large packet sizes
        //       We likely need to double-buffer the writes so we know to chunk

        // write out the packet header, leaving room for setting the packet length later

        let mut len_offset = 0;

        self.inner.write_with(
            PacketHeader {
                r#type: ty,
                status: Status::END_OF_MESSAGE,
                length: 0,
                server_process_id: 0,
                packet_id: 1,
            },
            &mut len_offset,
        );

        // write out the payload
        self.inner.write(payload);

        // overwrite the packet length now that we know it
        let len = self.inner.wbuf.len();
        self.inner.wbuf[len_offset..(len_offset + 2)].copy_from_slice(&(len as u16).to_be_bytes());
    }

    // receive the next packet from the database
    // blocks until a packet is available
    pub(super) async fn recv_packet(&mut self) -> Result<(PacketHeader, Bytes), Error> {
        // TODO: Support packet chunking for large packet sizes

        let header: PacketHeader = self.inner.read(8).await?;

        // NOTE: From what I can tell, the response type from the server should ~always~
        //       be TabularResult. Here we expect that and die otherwise.
        if !matches!(header.r#type, PacketType::TabularResult) {
            return Err(err_protocol!(
                "received unexpected packet: {:?}",
                header.r#type
            ));
        }

        let payload_len = (header.length - 8) as usize;
        let payload: Bytes = self.inner.read(payload_len).await?;

        Ok((header, payload))
    }

    // receive the next ~message~
    // TDS communicates in streams of packets that are themselves streams of messages
    pub(super) async fn recv_message(&mut self) -> Result<Message, Error> {
        loop {
            while self.response.as_ref().map_or(false, |r| !r.1.is_empty()) {
                let mut buf = if let Some((_, buf)) = self.response.as_mut() {
                    buf
                } else {
                    // this shouldn't be reachable but just nope out
                    // and head to refill our buffer
                    break;
                };

                let ty = MessageType::get(buf)?;
                let message = match ty {
                    MessageType::EnvChange => Message::EnvChange(EnvChange::get(buf)?),
                    MessageType::Info => Message::Info(Info::get(buf)?),
                    MessageType::Row => Message::Row(Row::get(buf, &self.columns)?),
                    MessageType::LoginAck => Message::LoginAck(LoginAck::get(buf)?),
                    MessageType::ReturnStatus => Message::ReturnStatus(ReturnStatus::get(buf)?),
                    MessageType::Done => Message::Done(Done::get(buf)?),
                    MessageType::DoneInProc => Message::DoneInProc(Done::get(buf)?),
                    MessageType::DoneProc => Message::DoneProc(Done::get(buf)?),

                    MessageType::Error => {
                        let err = ProtocolError::get(buf)?;
                        return Err(MsSqlDatabaseError(err).into());
                    }

                    MessageType::ColMetaData => {
                        // NOTE: there isn't anything to return as the data gets
                        //       consumed by the stream for use in subsequent Row decoding
                        ColMetaData::get(buf, &mut self.columns)?;
                        continue;
                    }
                };

                return Ok(message);
            }

            // no packet from the server to iterate (or its empty); fill our buffer
            self.response = Some(self.recv_packet().await?);
        }
    }
}

impl Deref for MsSqlStream {
    type Target = BufStream<MaybeTlsStream<TcpStream>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MsSqlStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
