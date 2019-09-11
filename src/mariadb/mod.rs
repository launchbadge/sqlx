// TODO: Remove after acitve development
#![allow(unused)]

mod row;
mod backend;
mod connection;
mod establish;
mod io;
mod protocol;
mod query;
pub mod types;

pub use self::{
    backend::MariaDb,
    connection::MariaDbRawConnection,
    query::MariaDbQueryParameters,
    row::MariaDbRow,
};

// pub use io::{BufExt, BufMutExt};
// pub use protocol::{
//     Capabilities, ColumnDefPacket, ColumnPacket, Decode, Encode, FieldDetailFlag, FieldType,
//     ProtocolType, ServerStatusFlag, SessionChangeType, StmtExecFlag,
// };

// 1) Get protocol compiling using io::Buf / io::BufMut
// 2) Switch MariaDbRawConnection to use io::BufStream
