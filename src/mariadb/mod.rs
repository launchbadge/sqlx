// TODO: Remove after acitve development
#![allow(ununsed)]

// mod backend;
// mod connection;
mod protocol;
mod io;
// mod query;
// pub mod types;

//pub use self::{
    // backend::MariaDb,
    // connection::MariaDbRawConnection,
    // query::MariaDbQueryParameters,
    // row::MariaDbRow,
//};

pub use io::{BufExt, BufMutExt};
pub use protocol::{Encode, Decode, Capabilities, FieldDetailFlag, FieldType, ProtocolType, ServerStatusFlag, SessionChangeType,
    StmtExecFlag, ColumnDefPacket, ColumnPacket
};


// 1) Get protocol compiling using io::Buf / io::BufMut
// 2) Switch MariaDbRawConnection to use io::BufStream
