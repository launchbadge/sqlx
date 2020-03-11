mod auth_plugin;
mod capabilities;
mod field;
mod status;
mod r#type;

pub(crate) use auth_plugin::AuthPlugin;
pub(crate) use capabilities::Capabilities;
pub(crate) use field::FieldFlags;
pub(crate) use r#type::TypeId;
pub(crate) use status::Status;

mod com_ping;
mod com_query;
mod com_stmt_execute;
mod com_stmt_prepare;
mod handshake;

pub(crate) use com_ping::ComPing;
pub(crate) use com_query::ComQuery;
pub(crate) use com_stmt_execute::{ComStmtExecute, Cursor};
pub(crate) use com_stmt_prepare::ComStmtPrepare;
pub(crate) use handshake::Handshake;

mod auth_switch;
mod column_count;
mod column_def;
mod com_stmt_prepare_ok;
mod eof;
mod err;
mod handshake_response;
mod ok;
mod row;
#[cfg_attr(not(feature = "tls"), allow(unused_imports, dead_code))]
mod ssl_request;

pub(crate) use auth_switch::AuthSwitch;
pub(crate) use column_count::ColumnCount;
pub(crate) use column_def::ColumnDefinition;
pub(crate) use com_stmt_prepare_ok::ComStmtPrepareOk;
pub(crate) use eof::EofPacket;
pub(crate) use err::ErrPacket;
pub(crate) use handshake_response::HandshakeResponse;
pub(crate) use ok::OkPacket;
pub(crate) use row::Row;
#[cfg_attr(not(feature = "tls"), allow(unused_imports, dead_code))]
pub(crate) use ssl_request::SslRequest;

pub(crate) trait Encode {
    fn encode(&self, buf: &mut Vec<u8>, capabilities: Capabilities);
}

impl Encode for &'_ [u8] {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        use crate::io::BufMut;

        buf.put_bytes(self);
    }
}
