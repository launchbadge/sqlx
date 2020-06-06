use bytes::{Buf, Bytes};

use crate::mssql::protocol::col_meta_data::ColMetaData;
use crate::mssql::protocol::done::Done;
use crate::mssql::protocol::env_change::EnvChange;
use crate::mssql::protocol::error::Error;
use crate::mssql::protocol::info::Info;
use crate::mssql::protocol::login_ack::LoginAck;
use crate::mssql::protocol::row::Row;

#[derive(Debug)]
pub(crate) enum Message {
    Info(Info),
    LoginAck(LoginAck),
    EnvChange(EnvChange),
    Done(Done),
    Row(Row),
}

#[derive(Debug)]
pub(crate) enum MessageType {
    Info,
    LoginAck,
    EnvChange,
    Done,
    Row,
    Error,
    ColMetaData,
}

impl MessageType {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, crate::error::Error> {
        Ok(match buf.get_u8() {
            0x81 => MessageType::ColMetaData,
            0xaa => MessageType::Error,
            0xab => MessageType::Info,
            0xad => MessageType::LoginAck,
            0xd1 => MessageType::Row,
            0xe3 => MessageType::EnvChange,
            0xfd => MessageType::Done,

            ty => {
                return Err(err_protocol!(
                    "unknown value `0x{:02x?}` for message type in token stream",
                    ty
                ));
            }
        })
    }
}
