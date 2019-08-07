pub mod connection;
pub mod protocol;

// Re-export all the things
pub use connection::{ConnContext, Connection, Framed};
pub use protocol::{
    AuthenticationSwitchRequestPacket, BufMut, Capabilities, ColumnDefPacket, ColumnPacket,
    ComDebug, ComInitDb, ComPing, ComProcessKill, ComQuery, ComQuit, ComResetConnection,
    ComSetOption, ComShutdown, ComSleep, ComStatistics, ComStmtClose, ComStmtExec, ComStmtFetch,
    ComStmtPrepare, ComStmtPrepareOk, ComStmtPrepareResp, DeContext, Decode, Decoder, Encode,
    EofPacket, ErrPacket, ErrorCode, FieldDetailFlag, FieldType, HandshakeResponsePacket,
    InitialHandshakePacket, OkPacket, PacketHeader, ResultRowText, ResultRowBinary, ResultRow, ResultSet, SSLRequestPacket,
    ServerStatusFlag, SessionChangeType, SetOptionOptions, ShutdownOptions, StmtExecFlag, ProtocolType
};
