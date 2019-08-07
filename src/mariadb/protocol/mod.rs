// Reference: https://mariadb.com/kb/en/library/connection
// Packets: https://mariadb.com/kb/en/library/0-packet

// TODO: Handle lengths which are greater than 3 bytes
// Either break the packet into several smaller ones, or
// return error
// TODO: Handle different Capabilities for server and client
// TODO: Handle when capability is set, but field is None

pub mod decode;
pub mod encode;
pub mod error_codes;
pub mod packets;
pub mod types;

// Re-export all the things
pub use packets::{
    AuthenticationSwitchRequestPacket, ColumnDefPacket, ColumnPacket, ComDebug, ComInitDb, ComPing,
    ComProcessKill, ComQuery, ComQuit, ComResetConnection, ComSetOption, ComShutdown, ComSleep,
    ComStatistics, ComStmtClose, ComStmtExec, ComStmtFetch, ComStmtPrepare, ComStmtPrepareOk,
    ComStmtPrepareResp, ComStmtReset, EofPacket, ErrPacket, HandshakeResponsePacket,
    InitialHandshakePacket, OkPacket, PacketHeader, ResultRowText, ResultRowBinary, ResultSet, SSLRequestPacket,
    SetOptionOptions, ShutdownOptions, ResultRow
};

pub use decode::{DeContext, Decode, Decoder};

pub use encode::{BufMut, Encode};

pub use error_codes::ErrorCode;

pub use types::{
    ProtocolType, Capabilities, FieldDetailFlag, FieldType, ServerStatusFlag, SessionChangeType, StmtExecFlag,
};
