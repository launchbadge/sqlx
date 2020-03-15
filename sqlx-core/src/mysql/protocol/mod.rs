// There is much to the protocol that is not yet used. As we mature we'll be trimming
// the size of this module to exactly what is necessary.
#![allow(unused)]

mod decode;
mod encode;

pub use decode::Decode;
pub use encode::Encode;

mod auth_plugin;
mod capabilities;
mod field;
mod status;
mod r#type;

pub use auth_plugin::AuthPlugin;
pub use capabilities::Capabilities;
pub use field::FieldFlags;
pub use r#type::TypeId;
pub use status::Status;

mod com_ping;
mod com_query;
mod com_set_option;
mod com_stmt_execute;
mod com_stmt_prepare;
mod handshake;

pub use com_ping::ComPing;
pub use com_query::ComQuery;
pub use com_set_option::{ComSetOption, SetOption};
pub use com_stmt_execute::{ComStmtExecute, Cursor};
pub use com_stmt_prepare::ComStmtPrepare;
pub use handshake::Handshake;

mod auth_switch;
mod column_count;
mod column_def;
mod com_stmt_prepare_ok;
mod eof;
mod err;
mod handshake_response;
mod ok;
mod row;
mod ssl_request;

pub use auth_switch::AuthSwitch;
pub use column_count::ColumnCount;
pub use column_def::ColumnDefinition;
pub use com_stmt_prepare_ok::ComStmtPrepareOk;
pub use eof::EofPacket;
pub use err::ErrPacket;
pub use handshake_response::HandshakeResponse;
pub use ok::OkPacket;
pub use row::Row;
pub use ssl_request::SslRequest;
