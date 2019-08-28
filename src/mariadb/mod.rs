pub mod backend;
pub mod connection;
pub mod protocol;
pub mod query;
pub mod types;

// Re-export all the things
pub use connection::{ConnContext, Framed, MariaDbRawConnection};
pub use protocol::{
    AuthenticationSwitchRequestPacket, BufMut, Capabilities, ColumnDefPacket, ColumnPacket,
    ComDebug, ComInitDb, ComPing, ComProcessKill, ComQuery, ComQuit, ComResetConnection,
    ComSetOption, ComShutdown, ComSleep, ComStatistics, ComStmtClose, ComStmtExec, ComStmtFetch,
    ComStmtPrepare, ComStmtPrepareOk, ComStmtPrepareResp, DeContext, Decode, Decoder, Encode,
    EofPacket, ErrPacket, ErrorCode, FieldDetailFlag, FieldType, HandshakeResponsePacket,
    InitialHandshakePacket, OkPacket, PacketHeader, ProtocolType, ResultRow, ResultRowBinary,
    ResultRowText, ResultSet, SSLRequestPacket, ServerStatusFlag, SessionChangeType,
    SetOptionOptions, ShutdownOptions, StmtExecFlag,
};

pub use backend::MariaDB;
