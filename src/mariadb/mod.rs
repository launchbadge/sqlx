pub mod connection;
pub mod protocol;
pub mod types;
pub mod backend;
pub mod query;

// Re-export all the things
pub use connection::{ConnContext, MariaDbRawConnection, Framed};
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
